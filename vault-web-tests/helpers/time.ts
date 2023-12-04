export async function sleep(durationMs: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, durationMs));
}

export async function waitFor(
  check: () => Promise<boolean>,
  timeoutMs = 500,
  sleepMs = 25,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;

  while (Date.now() < deadline) {
    if (await check()) {
      return;
    }

    await sleep(sleepMs);
  }

  throw new Error(`waitFor timeout in ${timeoutMs} ms`);
}
