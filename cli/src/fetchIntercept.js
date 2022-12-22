if(new URL(e.url).pathname.endsWith('/account')) {
  return {
    ...JSON.parse(t),
    subscription: {
      type: 'pro'
    },
    username: '🔓WeMod Pro Unlocker',
    profileImage: 'static/shared/images/default-profile-image.svg',
    ...{
       /*{%account%}*/
    }
  }
}