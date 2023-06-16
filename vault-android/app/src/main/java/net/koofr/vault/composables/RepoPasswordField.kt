package net.koofr.vault.composables

import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.VisualTransformation

@Composable
fun RepoPasswordField(
    value: TextFieldValue,
    onValueChange: (TextFieldValue) -> Unit,
    modifier: Modifier = Modifier,
    passwordVisible: Boolean,
    onPasswordVisibleChange: (Boolean) -> Unit,
    onDone: (() -> Unit)? = null,
    errorText: String? = null,
) {
    OutlinedTextField(
        value = value,
        onValueChange = onValueChange,
        modifier = modifier.semantics {
            this.contentDescription = "Safe Key"
        },
        singleLine = true,
        label = { Text("Safe Key") },
        placeholder = { Text("Safe Key") },
        visualTransformation = if (passwordVisible) VisualTransformation.None else PasswordVisualTransformation(),
        keyboardOptions = KeyboardOptions(
            keyboardType = KeyboardType.Password,
            imeAction = if (onDone != null) ImeAction.Done else ImeAction.Default,
        ),
        keyboardActions = KeyboardActions(
            onDone = onDone?.let {
                {
                    it()
                }
            },
        ),
        trailingIcon = {
            IconButton(onClick = { onPasswordVisibleChange(!passwordVisible) }) {
                Icon(
                    if (passwordVisible) {
                        Icons.Filled.Visibility
                    } else Icons.Filled.VisibilityOff,
                    if (passwordVisible) "Hide Safe Key" else "Show Safe Key",
                )
            }
        },
        isError = errorText != null,
        supportingText = errorText?.let {
            {
                Text(it, color = MaterialTheme.colorScheme.error)
            }
        },
    )
}
