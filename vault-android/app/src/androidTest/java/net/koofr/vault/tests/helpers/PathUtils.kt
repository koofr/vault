package net.koofr.vault.tests.helpers

sealed class PathError : Exception() {
    class RootPathNotAllowed : PathError()
    data class InvalidPath(val path: String) : PathError()
}

fun joinParentName(parentPath: String, name: String): String {
    return if (parentPath == "/") "/$name" else "$parentPath/$name"
}

fun splitParentName(path: String): Pair<String, String> {
    if (path == "/") {
        throw PathError.RootPathNotAllowed()
    }

    val parts = path.split("/").toMutableList()

    val name = parts.removeLastOrNull()
    if (name.isNullOrEmpty()) {
        throw PathError.InvalidPath(path)
    }

    var parentPath = parts.joinToString("/")
    if (parentPath.isEmpty()) {
        parentPath = "/"
    }

    return Pair(parentPath, name)
}
