<div align="center">
  <h1>WeMod Pro Unlocker</h1>
  <h4>This program patches the WeMod app to think you're a pro subscriber.</h4>
  <img width="256" src="https://user-images.githubusercontent.com/110846042/199363901-4b212629-895c-47a5-a059-4df198b64565.png" alt="WeMod Logo">
</div>

## ⚠️ DISCLAIMER: Eventhough I don't think it is likely, WeMod could in theory ban you for this. I take no responsibilty for any damage caused by the usage of this program.

<br/>

## ⬇️ Installation
#### Note: after following any of the steps below, you must restart WeMod (make sure to also close it from the tray)
For installation, you have three options.
1. Download a build from the releases tab
2. Install Cargo, then run
```
cargo install wemod-pro-unlocker
wemod-pro-unlocker
```
3. (Not recommended) Manually build it from source

<br/>

## 🔒 Is it safe?
This program may sound like maleware at first, but if you're unsure, just read the source code. It's quite small and (hopefully, I'm relatively new to Rust) readable.

<br/>

## 🏃‍♂️ Usage
| Argument                  	| Description                                                                                                 	| Example
|---------------------------	|-------------------------------------------------------------------------------------------------------------	|----------------------------------
| --wemod-dir <dir>         	| Path to your WeMod dir. By default, this is "%localappdata%/WeMod".                                         	| C:\WeMod
| --wemod-version <version> 	| The version to patch. By default, this will be the latest version installed. 	                                | 8.3.6
| --asar-bin <bin>          	| Path to the asar bin. By default, this will be "Program Files/nodejs/asar.cmd".                             	| C:\asar\asar.cmd
| -dev                      	| When used, WeMod will be patched to enable dev mode when opening it with "-dev".                             	| -----
| --account <json>            | Overwrites the account data. You can find all available options by searching for /v3/account in the dev tools | username:'myaccount',email:'test'

<br/>

## ❓ It stopped working after the latest update, what should I do?
This is probably because WeMod updated itself and now uses a new directory. To fix this, just run the program again.

##### Hint: You can create a shortcut which always launches both the unlocker and WeMod to automatically patch WeMod after updates.
