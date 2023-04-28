export function supportsRequestStreams() {
  if (!supportsReadableByteStream()) {
    return false;
  }

  let duplexAccessed = false;

  const hasContentType = new Request("", {
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

export function supportsReadableByteStream() {
  try {
    new ReadableStream({ type: "bytes" });

    return true;
  } catch {
    return false;
  }
}

export function sleep(durationMs) {
  return new Promise((resolve) => setTimeout(resolve, durationMs));
}
