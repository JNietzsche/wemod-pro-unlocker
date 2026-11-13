if ("application/json" === e.headers.get("Content-Type")) {
  var isAccount = false;
  try {
    isAccount = Boolean(
      e.url &&
        (e.url.includes('/account') ||
          new URL(e.url, "https://api.wemod.com").pathname.endsWith('/account'))
    );
  } catch (_) {}

  if (isAccount) {
    var originalData = await e.json();
    var PRO_MEMBER = 512;
    var WAS_EVER_PRO = 2048;
    var patchedFlags =
      (originalData && typeof originalData.flags === 'number'
        ? originalData.flags
        : 0) |
      PRO_MEMBER |
      WAS_EVER_PRO;
    return {
      ...originalData,
      flags: patchedFlags,
      subscription: {
        type: 'pro',
      },
      ...{
        /*{%account%}*/
        username: '🔓WeMod Pro',
      },
    };
  }
  return await e.json();
}
return await e.text();

