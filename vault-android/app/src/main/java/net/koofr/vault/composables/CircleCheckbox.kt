package net.koofr.vault.composables

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Circle
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.koofr.vault.ui.theme.VaultTheme

@Composable
fun CircleCheckbox(
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
    iconSize: Dp = 24.0.dp,
) {
    val imageVector = if (checked) Icons.Filled.CheckCircle else Icons.Filled.Circle
    val tint = MaterialTheme.colorScheme.primary

    IconButton(
        onClick = { onCheckedChange(!checked) },
        modifier = modifier,
    ) {
        Icon(
            imageVector = imageVector,
            tint = tint,
            modifier = Modifier
                .size(iconSize),
            contentDescription = "Checkbox",
        )
    }
}

@Preview(showBackground = true)
@Composable
fun CircleCheckboxPreview() {
    VaultTheme {
        Column {
            CircleCheckbox(checked = false, onCheckedChange = { })
            CircleCheckbox(checked = true, onCheckedChange = { })
        }
    }
}
