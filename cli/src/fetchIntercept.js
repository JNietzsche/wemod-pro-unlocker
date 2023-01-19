if ("application/json" === e.headers.get("Content-Type")) {
  if (new URL(e.url).pathname.endsWith('/account')) {
    var originalData = await e.json();
    return {
      ...originalData,
      subscription: {
        type: 'pro'
      },
      ...{
        /*{%account%}*/
        username: '🔓WeMod Pro Unlocker'
      }
    }
  }
  return e.json()
}
return e.text()