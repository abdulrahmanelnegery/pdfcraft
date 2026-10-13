package dev.pdfcraftenhanced;

import android.app.NativeActivity;
import android.content.ClipData;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.provider.OpenableColumns;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.ArrayList;
import java.util.List;

/**
 * The activity of PdfCraft Enhanced. It is a NativeActivity (the app is Rust) plus the one thing
 * that needs Java: the system document picker, whose answer comes back as an activity result.
 */
public class MainActivity extends NativeActivity {
    private static final int REQUEST_OPEN = 7001;

    /** Show the system picker for documents. Called from native code on any thread. */
    public void openDocuments(final boolean multiple) {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                intent.addCategory(Intent.CATEGORY_OPENABLE);
                intent.setType("*/*");
                intent.putExtra(Intent.EXTRA_MIME_TYPES, new String[] {"application/pdf", "image/*", "text/plain"});
                intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple);
                try {
                    startActivityForResult(intent, REQUEST_OPEN);
                } catch (RuntimeException e) {
                    nativePicked(new String[0]);
                }
            }
        });
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode != REQUEST_OPEN) {
            return;
        }
        final List<Uri> uris = new ArrayList<Uri>();
        if (resultCode == RESULT_OK && data != null) {
            ClipData clip = data.getClipData();
            if (clip != null) {
                for (int i = 0; i < clip.getItemCount(); i++) {
                    uris.add(clip.getItemAt(i).getUri());
                }
            } else if (data.getData() != null) {
                uris.add(data.getData());
            }
        }
        // Copy off the UI thread: a document can be large.
        new Thread(new Runnable() {
            @Override
            public void run() {
                List<String> paths = new ArrayList<String>();
                for (Uri uri : uris) {
                    String path = copyToCache(uri);
                    if (path != null) {
                        paths.add(path);
                    }
                }
                nativePicked(paths.toArray(new String[0]));
            }
        }).start();
    }

    /** The picked document as a file the app can read by path. */
    private String copyToCache(Uri uri) {
        String name = displayName(uri);
        File dir = new File(getCacheDir(), "picked");
        if (!dir.isDirectory() && !dir.mkdirs()) {
            return null;
        }
        File target = new File(dir, name);
        try (InputStream in = getContentResolver().openInputStream(uri); OutputStream out = new FileOutputStream(target)) {
            if (in == null) {
                return null;
            }
            byte[] buffer = new byte[64 * 1024];
            int n;
            while ((n = in.read(buffer)) > 0) {
                out.write(buffer, 0, n);
            }
        } catch (Exception e) {
            return null;
        }
        return target.getAbsolutePath();
    }

    private String displayName(Uri uri) {
        String name = null;
        Cursor cursor = getContentResolver().query(uri, new String[] {OpenableColumns.DISPLAY_NAME}, null, null, null);
        if (cursor != null) {
            try {
                if (cursor.moveToFirst()) {
                    name = cursor.getString(0);
                }
            } finally {
                cursor.close();
            }
        }
        if (name == null || name.isEmpty()) {
            name = "document.pdf";
        }
        // A file name only: no folders, and nothing that climbs out of the cache folder.
        name = name.replace('/', '_').replace('\\', '_');
        return name.equals(".") || name.equals("..") ? "document.pdf" : name;
    }

    /** Answered to the app: the picked files' paths, none if the user backed out. */
    private static native void nativePicked(String[] paths);
}
