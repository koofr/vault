package net.koofr.vault.utils

import android.app.Activity
import android.content.Context
import android.content.Intent
import androidx.activity.result.contract.ActivityResultContract
import androidx.annotation.CallSuper

object CustomActivityResultContracts {
    class GetContent : ActivityResultContract<Unit, Intent?>() {
        @CallSuper
        override fun createIntent(context: Context, input: Unit): Intent {
            return Intent(Intent.ACTION_GET_CONTENT)
                .addCategory(Intent.CATEGORY_OPENABLE)
                .setType("*/*")
                .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                .putExtra(Intent.EXTRA_ALLOW_MULTIPLE, true)
        }

        final override fun getSynchronousResult(
            context: Context,
            input: Unit,
        ): SynchronousResult<Intent?>? = null

        final override fun parseResult(resultCode: Int, intent: Intent?): Intent? {
            return intent.takeIf { resultCode == Activity.RESULT_OK }
        }
    }

    class OpenDocumentTree : ActivityResultContract<Unit, Intent?>() {
        @CallSuper
        override fun createIntent(context: Context, input: Unit): Intent {
            return Intent(Intent.ACTION_OPEN_DOCUMENT_TREE)
                .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }

        final override fun getSynchronousResult(
            context: Context,
            input: Unit,
        ): SynchronousResult<Intent?>? = null

        final override fun parseResult(resultCode: Int, intent: Intent?): Intent? {
            return intent.takeIf { resultCode == Activity.RESULT_OK }
        }
    }
}
