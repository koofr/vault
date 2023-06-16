package net.koofr.vault

import android.os.Build
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.koofr.vault.ui.theme.VaultTheme

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NotSupported() {
    val context = LocalContext.current

    @Suppress("DEPRECATION")
    val text =
        "${context.resources.getString(R.string.app_name)} app version that you've installed does not contain the native library for your architecture (${Build.CPU_ABI}). Please make sure you've installed the correct version."

    VaultTheme {
        Scaffold(topBar = {
            TopAppBar(title = {
                Text(text = "Vault")
            })
        }) { paddingValues ->
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text(
                    text,
                    style = MaterialTheme.typography.titleMedium.copy(textAlign = TextAlign.Center),
                    modifier = Modifier.padding(20.dp),
                )
            }
        }
    }
}
