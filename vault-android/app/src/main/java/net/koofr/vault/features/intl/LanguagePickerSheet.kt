package net.koofr.vault.features.intl

import android.app.Activity
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.selection.toggleable
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import net.koofr.vault.IntlLocale
import net.koofr.vault.R
import net.koofr.vault.features.mobilevault.subscribe
import net.koofr.vault.ui.theme.KoofrGreen

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LanguagePickerSheet(
    intlHelper: IntlHelper,
    onDismiss: () -> Unit,
) {
    val activity = LocalContext.current as Activity

    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()

    val locales = subscribe(
        { v, cb -> v.intlLocalesSubscribe(cb = cb) },
        { v, id -> v.intlLocalesData(id = id) },
    )

    val currentLocale = subscribe(
        { v, cb -> v.intlCurrentLocaleSubscribe(cb = cb) },
        { v, id -> v.intlCurrentLocaleData(id = id) },
    )

    ModalBottomSheet(onDismissRequest = onDismiss, sheetState = sheetState) {
        LazyColumn(
            modifier = Modifier
                .padding(20.dp, 0.dp, 20.dp, 20.dp)
                .fillMaxWidth(),
        ) {
            item {
                Text(
                    stringResource(R.string.language_picker_title),
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.padding(bottom = 10.dp),
                )
            }

            val localesList = locales.value ?: emptyList()
            val selectedLocale = currentLocale.value?.locale

            items(localesList, key = { it.locale }) { item ->
                LanguageRow(
                    item = item,
                    isActive = item.locale == selectedLocale,
                    onSelect = {
                        intlHelper.changeLocale(item.locale)

                        scope.launch {
                            sheetState.hide()

                            onDismiss()
                        }

                        activity.recreate()
                    },
                )
            }
        }
    }
}

@Composable
private fun LanguageRow(
    item: IntlLocale,
    isActive: Boolean,
    onSelect: () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(45.dp)
            .toggleable(
                value = isActive,
                role = Role.Checkbox,
                onValueChange = { onSelect() },
            )
            .padding(start = 8.dp, end = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = item.name,
            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
            modifier = Modifier
                .weight(1f)
                .padding(end = 10.dp),
            color = if (isActive) KoofrGreen else Color.Unspecified,
        )

        if (isActive) {
            Icon(Icons.Filled.Check, null, tint = KoofrGreen)
        }
    }
}
