;(()=>{
  console.info("WeMod Pro Unlocker v/*{%version%}*/")

  let trig = () => {
    document.querySelectorAll('promotion-banner').forEach(element => element.remove());
    document.querySelectorAll('.cheats-wrapper button.pro-upgrade').forEach(element => element.remove());
  };

  setInterval(() => trig(), 2000);
  trig();
})();