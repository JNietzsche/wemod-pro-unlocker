if ("application/json" === e.headers.get("Content-Type")) {
  if (new URL(e.url).pathname.endsWith('/account')) {
    var originalData = await e.json();
    var PRO_MEMBER = 512;
    var WAS_EVER_PRO = 2048;
    var CREATOR = 64;
    var patchedFlags = (originalData && typeof originalData.flags === 'number' ? originalData.flags : 0) | PRO_MEMBER | WAS_EVER_PRO | CREATOR;
    return {
      ...originalData,
      flags: patchedFlags,
      subscription: {
        type: 'pro'
      },
      ...{
        /*{%account%}*/
        username: '🔓WeMod Pro'
      }
    }
  }
  return e.json()
}
return e.text()
