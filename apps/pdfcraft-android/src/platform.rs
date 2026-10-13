//! Calls into the Android activity through JNI: the PDF an `ACTION_VIEW` intent carries, and the
//! storage permission the in-app file browser needs.

use android_activity::AndroidApp;
use jni::errors::Result as JniResult;
use jni::objects::{JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};

/// A document handed over by another app (a file manager, a download, a mail attachment).
pub struct Incoming {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// Larger than any PDF worth opening on a phone; a stream past it is dropped, not read.
const MAX_BYTES: usize = 1 << 30;

fn with_activity<R>(app: &AndroidApp, f: impl FnOnce(&mut JNIEnv, &JObject) -> JniResult<R>) -> Option<R> {
    // SAFETY: android-activity keeps the VM and the activity object valid for as long as the app runs.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.ok()?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    let mut env = vm.attach_current_thread().ok()?;
    let out = f(&mut env, &activity);
    if out.is_err() {
        // A Java exception left pending would abort the next JNI call.
        let _ = env.exception_clear();
    }
    out.map_err(|e| log::warn!("android call failed: {e}")).ok()
}

fn string_of(env: &mut JNIEnv, object: JObject) -> JniResult<String> {
    Ok(env.get_string(&JString::from(object))?.into())
}

/// The document of the intent that started the activity, when it is a PDF `ACTION_VIEW`.
pub fn incoming_document(app: &AndroidApp) -> Option<Incoming> {
    with_activity(app, |env, activity| {
        let intent = env.call_method(activity, "getIntent", "()Landroid/content/Intent;", &[])?.l()?;
        if intent.is_null() {
            return Ok(None);
        }
        let action = env.call_method(&intent, "getAction", "()Ljava/lang/String;", &[])?.l()?;
        if action.is_null() || string_of(env, action)? != "android.intent.action.VIEW" {
            return Ok(None);
        }
        let uri = env.call_method(&intent, "getData", "()Landroid/net/Uri;", &[])?.l()?;
        if uri.is_null() {
            return Ok(None);
        }
        let resolver = env.call_method(activity, "getContentResolver", "()Landroid/content/ContentResolver;", &[])?.l()?;
        let stream = env.call_method(&resolver, "openInputStream", "(Landroid/net/Uri;)Ljava/io/InputStream;", &[JValue::Object(&uri)])?.l()?;
        if stream.is_null() {
            return Ok(None);
        }
        let buffer = env.new_byte_array(64 * 1024)?;
        let mut bytes = Vec::new();
        loop {
            let n = env.call_method(&stream, "read", "([B)I", &[JValue::Object(&buffer)])?.i()?;
            let Ok(n) = usize::try_from(n) else { break };
            if n == 0 {
                break;
            }
            let mut chunk = vec![0i8; n];
            env.get_byte_array_region(&buffer, 0, &mut chunk)?;
            bytes.extend(chunk.iter().map(|b| *b as u8));
            if bytes.len() > MAX_BYTES {
                let _ = env.call_method(&stream, "close", "()V", &[]);
                return Ok(None);
            }
        }
        env.call_method(&stream, "close", "()V", &[])?;
        let name = display_name(env, &resolver, &uri).unwrap_or_else(|| "document.pdf".to_string());
        Ok(Some(Incoming { name, bytes }))
    })
    .flatten()
}

/// The file name the provider reports for `uri`, if it reports one.
fn display_name(env: &mut JNIEnv, resolver: &JObject, uri: &JObject) -> Option<String> {
    let name = (|| -> JniResult<Option<String>> {
        let column = env.new_string("_display_name")?;
        let columns = env.new_object_array(1, "java/lang/String", &column)?;
        let null = JObject::null();
        let cursor = env
            .call_method(
                resolver,
                "query",
                "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
                &[JValue::Object(uri), JValue::Object(&columns), JValue::Object(&null), JValue::Object(&null), JValue::Object(&null)],
            )?
            .l()?;
        if cursor.is_null() {
            return Ok(None);
        }
        let mut found = None;
        if env.call_method(&cursor, "moveToFirst", "()Z", &[])?.z()? {
            let value = env.call_method(&cursor, "getString", "(I)Ljava/lang/String;", &[JValue::Int(0)])?.l()?;
            if !value.is_null() {
                found = Some(string_of(env, value)?);
            }
        }
        env.call_method(&cursor, "close", "()V", &[])?;
        Ok(found)
    })();
    if name.is_err() {
        let _ = env.exception_clear();
    }
    name.ok().flatten().filter(|n| !n.is_empty())
}

fn sdk_int(env: &mut JNIEnv) -> JniResult<i32> {
    let version = env.find_class("android/os/Build$VERSION")?;
    env.get_static_field(version, "SDK_INT", "I")?.i()
}

/// Whether the app may read shared storage: "All files access" on Android 11 and newer, the classic
/// read permission before that.
pub fn has_storage_access(app: &AndroidApp) -> bool {
    with_activity(app, |env, activity| {
        if sdk_int(env)? >= 30 {
            return env.call_static_method("android/os/Environment", "isExternalStorageManager", "()Z", &[])?.z();
        }
        let permission = env.new_string("android.permission.READ_EXTERNAL_STORAGE")?;
        let granted = env.call_method(activity, "checkSelfPermission", "(Ljava/lang/String;)I", &[JValue::Object(&permission)])?.i()?;
        Ok(granted == 0)
    })
    .unwrap_or(false)
}

/// Ask for storage access: the "All files access" settings page of this app on Android 11 and newer,
/// the permission prompt before that.
pub fn request_storage_access(app: &AndroidApp) {
    let _ = with_activity(app, |env, activity| {
        if sdk_int(env)? >= 30 {
            let package = env.call_method(activity, "getPackageName", "()Ljava/lang/String;", &[])?.l()?;
            let package = format!("package:{}", string_of(env, package)?);
            let package = env.new_string(package)?;
            let uri =
                env.call_static_method("android/net/Uri", "parse", "(Ljava/lang/String;)Landroid/net/Uri;", &[JValue::Object(&package)])?.l()?;
            let action = env.new_string("android.settings.MANAGE_APP_ALL_FILES_ACCESS_PERMISSION")?;
            let intent =
                env.new_object("android/content/Intent", "(Ljava/lang/String;Landroid/net/Uri;)V", &[JValue::Object(&action), JValue::Object(&uri)])?;
            env.call_method(activity, "startActivity", "(Landroid/content/Intent;)V", &[JValue::Object(&intent)])?;
        } else {
            let read = env.new_string("android.permission.READ_EXTERNAL_STORAGE")?;
            let write = env.new_string("android.permission.WRITE_EXTERNAL_STORAGE")?;
            let list = env.new_object_array(2, "java/lang/String", &read)?;
            env.set_object_array_element(&list, 1, &write)?;
            env.call_method(activity, "requestPermissions", "([Ljava/lang/String;I)V", &[JValue::Object(&list), JValue::Int(1)])?;
        }
        Ok(())
    });
}
