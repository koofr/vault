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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.LocalSnackbarHostState
import net.koofr.vault.MobileVault
import net.koofr.vault.R
import net.koofr.vault.features.auth.AuthHelper
import net.koofr.vault.features.navigation.LocalNavController
import net.koofr.vault.ui.theme.KoofrBlue
import javax.inject.Inject

@HiltViewModel
class LandingScreenViewModel @Inject constructor(
    val mobileVault: MobileVault,
    val authHelper: AuthHelper,
) : ViewModel()

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LandingScreen(vm: LandingScreenViewModel = hiltViewModel()) {
    val context = LocalContext.current
    val navController = LocalNavController.current

    Scaffold(topBar = {
        TopAppBar(title = {}, actions = {
            IconButton(onClick = {
                navController.navigate("info")
            }) {
                Icon(Icons.Outlined.Info, "Info")
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
                    contentDescription = "${context.resources.getString(R.string.app_name)} logo",
                    modifier = Modifier.padding(bottom = 50.dp),
                )

                Text(
                    "One vault for all\nyour private files.",
                    style = TextStyle(
                        fontFamily = FontFamily.Default,
                        fontWeight = FontWeight.Bold,
                        fontSize = 32.sp,
                        textAlign = TextAlign.Center,
                    ),
                    modifier = Modifier.padding(bottom = 32.dp),
                )

                Text(
                    "Powerful, open source client-side encryption. Unlock enhanced security for your most sensitive files.",
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
                    contentDescription = "Graphic",
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
                        "Get started",
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
    }
}
