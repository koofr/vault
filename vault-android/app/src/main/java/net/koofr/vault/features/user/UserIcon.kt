package net.koofr.vault.features.user

import android.graphics.BitmapFactory
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.koofr.vault.MobileVault
import net.koofr.vault.features.mobilevault.subscribe
import javax.inject.Inject

@HiltViewModel
class UserIconViewModel @Inject constructor(
    val mobileVault: MobileVault,
) : ViewModel() {
    val profilePicture = mutableStateOf<ImageBitmap?>(null)

    fun loadProfilePicture() {
        profilePicture.value = mobileVault.userGetProfilePicture()?.let { bytes ->
            val bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.size)

            bitmap.asImageBitmap()
        }
    }
}

@Composable
fun UserIcon(vm: UserIconViewModel = hiltViewModel()) {
    val userProfilePictureLoaded = subscribe(
        { v, cb -> v.userProfilePictureLoadedSubscribe(cb) },
        { v, id -> v.userProfilePictureLoadedData(id) },
    )

    val user = subscribe(
        { v, cb -> v.userSubscribe(cb) },
        { v, id -> v.userData(id) },
    )

    LaunchedEffect(Unit) {
        vm.mobileVault.userEnsureProfilePicture()
    }

    LaunchedEffect(userProfilePictureLoaded.value) {
        vm.loadProfilePicture()
    }

    if (userProfilePictureLoaded.value != true) {
        Box(
            modifier = Modifier
                .width(40.dp)
                .height(40.dp),
        )
    } else if (vm.profilePicture.value != null) {
        Image(
            vm.profilePicture.value!!,
            null,
            contentScale = ContentScale.Crop,
            modifier = Modifier
                .width(40.dp)
                .height(40.dp)
                .clip(CircleShape),
        )
    } else {
        UserIconFallback(name = user.value?.fullName)
    }
}

@Composable
fun UserIconFallback(name: String?) {
    Box(
        modifier = Modifier
            .width(40.dp)
            .height(40.dp)
            .clip(CircleShape),
    ) {
        Row(
            modifier = Modifier
                .width(40.dp)
                .height(40.dp)
                .background(Color(0xFFD4D6D7))
                .clip(CircleShape),
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                name?.firstOrNull()?.uppercase() ?: "",
                fontWeight = FontWeight.Bold,
                color = Color.White,
            )
        }
    }
}

@Preview(showBackground = true)
@Composable
fun UserIconFallbackPreview() {
    UserIconFallback(name = "Test User")
}
