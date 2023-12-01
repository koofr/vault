package net.koofr.vault.features.repo

import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.PointerEventTimeoutCancellationException
import androidx.compose.ui.input.pointer.pointerInput
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.coroutineScope
import net.koofr.vault.MobileVault
import javax.inject.Inject

@HiltViewModel
class UnlockedRepoWrapperViewModel @Inject constructor(
    val mobileVault: MobileVault,
) : ViewModel()

@Composable
fun UnlockedRepoWrapper(
    repoId: String,
    vm: UnlockedRepoWrapperViewModel = hiltViewModel(),
    content: @Composable () -> Unit,
) {
    // lock the safe box on 5 second long press
    Box(
        modifier = Modifier
            .pointerInput(Unit) {
                coroutineScope {
                    awaitEachGesture {
                        awaitFirstDown(requireUnconsumed = false)

                        vm.mobileVault.reposTouchRepo(repoId = repoId)

                        try {
                            withTimeout(5000) {
                                waitForUpOrCancellation()
                            }
                        } catch (_: PointerEventTimeoutCancellationException) {
                            vm.mobileVault.reposLockRepo(repoId = repoId)
                        }
                    }
                }
            },
    ) {
        content()
    }
}
