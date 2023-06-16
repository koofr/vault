package net.koofr.vault.composables

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FormInfoSheet(
    title: String,
    text: String,
    isVisible: MutableState<Boolean>,
) {
    if (isVisible.value) {
        ModalBottomSheet(onDismissRequest = {
            isVisible.value = false
        }, sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)) {
            Column(Modifier.padding(bottom = 40.dp)) {
                Row(
                    Modifier
                        .padding(start = 16.dp, end = 16.dp, bottom = 10.dp)
                        .fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        title,
                        style = MaterialTheme.typography.titleMedium,
                    )
                }

                Column(
                    Modifier
                        .weight(weight = 1f, fill = false)
                        .padding(start = 16.dp, end = 16.dp, bottom = 16.dp),
                ) {
                    Text(text)
                }
            }
        }
    }
}
