package net.koofr.vault.composables

// based on https://github.com/leinardi/FloatingActionButtonSpeedDial

import android.annotation.SuppressLint
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.FiniteAnimationSpec
import androidx.compose.animation.core.MutableTransitionState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SmallFloatingActionButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import net.koofr.vault.ui.theme.VaultTheme

data class MultiAddButtonItem(
    val text: String,
    val onClick: () -> Unit,
)

@Composable
fun MultiAddButton(items: List<MultiAddButtonItem>) {
    val animVisibleState = remember { MutableTransitionState(false) }
    val rotate = animateFloatAsState(if (animVisibleState.targetState) 45f else 0f)

    val mainSize = 56.dp
    val popupOffset =
        LocalDensity.current.run { IntOffset(mainSize.roundToPx(), -mainSize.roundToPx()) }

    val onDismissPopup = remember {
        {
            animVisibleState.targetState = false
        }
    }

    Column(modifier = Modifier.wrapContentSize()) {
        Column(modifier = Modifier.wrapContentSize()) {
            FloatingActionButton(
                onClick = {
                    if (!animVisibleState.currentState && !animVisibleState.targetState) {
                        animVisibleState.targetState = true
                    }
                },
                containerColor = MaterialTheme.colorScheme.primary,
                shape = CircleShape,
                modifier = Modifier.size(mainSize),
            ) {
                Icon(
                    Icons.Filled.Add,
                    "Add",
                    modifier = Modifier
                        .rotate(rotate.value),
                )
            }
        }

        AnimatedVisibility(
            visibleState = animVisibleState,
            enter = fadeIn(tween(durationMillis = 0)),
            exit = fadeOut(animationSpec = tween(durationMillis = 200)),
        ) {
            Popup(
                alignment = Alignment.BottomEnd,
                offset = popupOffset,
                onDismissRequest = onDismissPopup,
                properties = PopupProperties(focusable = true),
            ) {
                Column(modifier = Modifier.wrapContentSize(), horizontalAlignment = Alignment.End) {
                    items.forEachIndexed { index, item ->
                        MultiAddButtonView(
                            item,
                            animVisibleState,
                            items.size,
                            index,
                            mainSize,
                            onDismissPopup,
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun MultiAddButtonView(
    item: MultiAddButtonItem,
    visibleState: MutableTransitionState<Boolean>,
    count: Int,
    index: Int,
    mainSize: Dp,
    onDismissPopup: () -> Unit,
) {
    val onClick = remember {
        {
            onDismissPopup()
            item.onClick()
        }
    }

    val itemSize = 40.dp
    val itemPaddingEnd = mainSize / 2 - itemSize / 2

    val interactionSource = remember { MutableInteractionSource() }

    val contentAnimationDelayInMillis = 20
    val animationInSpecIntOffset: FiniteAnimationSpec<IntOffset> =
        tween(delayMillis = contentAnimationDelayInMillis * (count - index))
    val animationInSpecFloat: FiniteAnimationSpec<Float> =
        tween(delayMillis = contentAnimationDelayInMillis * (count - index))
    val animationOutSpecIntOffset: FiniteAnimationSpec<IntOffset> =
        tween(delayMillis = contentAnimationDelayInMillis * index)
    val animationOutSpecFloat: FiniteAnimationSpec<Float> =
        tween(delayMillis = contentAnimationDelayInMillis * index)

    AnimatedVisibility(
        visibleState = visibleState,
        enter = fadeIn(animationInSpecFloat) + slideInVertically(animationInSpecIntOffset) { it / 2 },
        exit = fadeOut(animationOutSpecFloat) + slideOutVertically(animationOutSpecIntOffset) { it / 2 },
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier
                .padding(end = itemPaddingEnd, bottom = 20.dp),
        ) {
            Surface(
                onClick = onClick,
                color = MaterialTheme.colorScheme.tertiary,
                shape = RoundedCornerShape(12.0.dp),
                modifier = Modifier.semantics { role = Role.Button },
                tonalElevation = 1.0.dp,
                shadowElevation = 1.0.dp,
                interactionSource = interactionSource,
            ) {
                Text(
                    item.text,
                    fontSize = 13.sp,
                    color = Color.White,
                    modifier = Modifier
                        .padding(start = 10.dp, end = 10.dp, top = 5.dp, bottom = 5.dp),
                )
            }
            Spacer(modifier = Modifier.width(16.dp))
            SmallFloatingActionButton(
                modifier = Modifier.size(40.dp),
                shape = CircleShape,
                containerColor = MaterialTheme.colorScheme.tertiary,
                onClick = onClick,
                interactionSource = interactionSource,
            ) {
                Icon(
                    Icons.Filled.Add,
                    item.text,
                )
            }
        }
    }
}

@SuppressLint("UnusedMaterial3ScaffoldPaddingParameter")
@Preview(showBackground = true)
@Composable
fun MultiAddButtonPreview() {
    VaultTheme {
        Scaffold(floatingActionButton = {
            MultiAddButton(
                listOf(
                    MultiAddButtonItem("Item 1") {},
                    MultiAddButtonItem("Item 2") {},
                ),
            )
        }, content = {
        })
    }
}
