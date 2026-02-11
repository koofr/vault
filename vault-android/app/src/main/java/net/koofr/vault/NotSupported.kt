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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.koofr.vault.ui.theme.VaultTheme

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NotSupported() {
    val context = LocalContext.current

    @Suppress("DEPRECATION")
    val cpuAbi = Build.CPU_ABI

    VaultTheme {
        Scaffold(topBar = {
            TopAppBar(title = {
                Text(text = stringResource(R.string.not_supported_title))
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
                    stringResource(
                        R.string.not_supported_message,
                        cpuAbi,
                    ),
                    style = MaterialTheme.typography.titleMedium.copy(textAlign = TextAlign.Center),
                    modifier = Modifier.padding(20.dp),
                )
            }
        }
    }
}
