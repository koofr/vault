package net.koofr.vault.features.landing

import androidx.compose.foundation.Image
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Language
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.R
import net.koofr.vault.features.auth.AuthHelper
import net.koofr.vault.features.intl.IntlHelper
import net.koofr.vault.features.intl.LanguagePickerSheet
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.ui.theme.KoofrBlue
import javax.inject.Inject

@HiltViewModel
class LandingScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    val authHelper: AuthHelper,
    val intlHelper: IntlHelper,
) : ViewModel()

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LandingScreen(vm: LandingScreenViewModel = hiltViewModel()) {
    val context = LocalContext.current
    val navController = LocalNavController.current
    val languagePickerVisible = remember { mutableStateOf(false) }

    Scaffold(topBar = {
        TopAppBar(title = {}, actions = {
            IconButton(onClick = {
                languagePickerVisible.value = true
            }) {
                Icon(
                    Icons.Outlined.Language,
                    stringResource(R.string.landing_language_picker_button_content_desc),
                )
            }
            IconButton(onClick = {
                navController.navigate("info")
            }) {
                Icon(Icons.Outlined.Info, stringResource(R.string.landing_info_button_content_desc))
            }
        })
    }, snackbarHost = { SnackbarHost(LocalSnackbarHostState.current) }) { paddingValues ->
        Column(
            modifier = Modifier
                .padding(paddingValues)
                .fillMaxSize(),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Column(
                modifier = Modifier.padding(top = 0.dp, start = 20.dp, end = 20.dp),
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Image(
                    painter = painterResource(id = if (isSystemInDarkTheme()) R.drawable.landing_logo_dark else R.drawable.landing_logo),
                    contentDescription = stringResource(R.string.landing_logo_content_desc),
                    modifier = Modifier.padding(bottom = 50.dp),
                )

                Text(
                    stringResource(R.string.landing_title),
                    style = TextStyle(
                        fontFamily = FontFamily.Default,
                        fontWeight = FontWeight.Bold,
                        fontSize = 32.sp,
                        textAlign = TextAlign.Center,
                    ),
                    modifier = Modifier.padding(bottom = 32.dp),
                )

                Text(
                    stringResource(R.string.landing_message),
                    style = TextStyle(
                        fontFamily = FontFamily.Default,
                        fontWeight = FontWeight.Normal,
                        fontSize = 18.sp,
                        textAlign = TextAlign.Center,
                    ),
                    modifier = Modifier
                        .padding(bottom = 30.dp)
                        .width(300.dp),
                )

                Image(
                    painter = painterResource(id = if (isSystemInDarkTheme()) R.drawable.landing_graphic_dark else R.drawable.landing_graphic),
                    contentDescription = stringResource(R.string.landing_graphic_content_desc),
                    modifier = Modifier
                        .padding(bottom = 40.dp)
                        .weight(1f, fill = false),
                )

                Button(
                    onClick = {
                        vm.authHelper.login(context)
                    },
                    colors = ButtonDefaults.buttonColors(containerColor = KoofrBlue),
                    shape = RoundedCornerShape(3.dp),
                    modifier = Modifier
                        .width(300.dp)
                        .height(60.dp)
                        .padding(bottom = 10.dp),
                ) {
                    Text(
                        stringResource(R.string.landing_get_started_button),
                        color = Color.White,
                        style = TextStyle(
                            fontFamily = FontFamily.Default,
                            fontWeight = FontWeight.Bold,
                            fontSize = 20.sp,
                        ),
                    )
                }
            }
        }

        if (languagePickerVisible.value) {
            LanguagePickerSheet(
                intlHelper = vm.intlHelper,
                onDismiss = {
                    languagePickerVisible.value = false
                },
            )
        }
    }
}
