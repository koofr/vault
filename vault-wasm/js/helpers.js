export function supportsRequestStreams() {
  if (!supportsReadableByteStream()) {
    return false;
  }

  let duplexAccessed = false;

  const hasContentType = new Request("http://dummy", {
    body: new ReadableStream(),
    method: "POST",
    get duplex() {
      duplexAccessed = true;
      return "half";
    },
  }).headers.has("Content-Type");

  return duplexAccessed && !hasContentType;
}

export function streamToBlob(stream, contentTypeOpt) {
  const headers = {};
  if (contentTypeOpt !== undefined) {
    headers["Content-Type"] = contentTypeOpt;
  }

  const r = new Response(stream, {
    headers,
  });

  return r.blob();
}

// Checks that ReadableStream supports BYOB reader (works in Chrome and Firefox,
// does not work in Safari)
export function supportsReadableByteStream() {
  try {
    // Create a minimal byte stream
    const stream = new ReadableStream({ type: "bytes" });

    // Try to get a BYOB reader. This throws a TypeError in browsers that don't
    // support BYOB.
    stream.getReader({ mode: "byob" }).releaseLock();

    return true;
  } catch {
    return false;
  }
}

export function errorString(err) {
  try {
    let s = err.message != null ? err.message : `${err}`;

    if (err.cause != null) {
      s = `${s}: ${errorString(err.cause)}`;
    }

    return s;
  } catch {
    return "unknown error";
  }
}
