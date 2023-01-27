<div align="center">
  <h1>WeMod Pro Unlocker - GUI</h1>
  <h4>This program patches the WeMod app to think you're a pro subscriber.</h4>
  <img src="https://img.shields.io/github/downloads/bennett-sh/wemod-pro-unlocker/total" alt="Downloads on GitHub">
  <img src="https://img.shields.io/github/v/release/bennett-sh/wemod-pro-unlocker.svg" alt="Latest version">
  <img src="https://img.shields.io/crates/l/wemod-pro-unlocker?color=green" alt="License">


  <img src="https://img.shields.io/badge/.NET-512BD4?logo=dotnet" alt=".NET">
  <img src="https://img.shields.io/badge/Windows-10+11-0078D4?logo=windows-11" alt="Windows 10 and 11">
  <img src="https://img.shields.io/github/languages/code-size/bennett-sh/wemod-pro-unlocker?color=yellow" alt="Code Size"><br/><br/>
  <img width="256" src="https://user-images.githubusercontent.com/110846042/204567385-4df3007c-7a63-40fd-9feb-f9f36aa43030.png" alt="WeMod Pro Unlocker Logo">
</div>

#### [Back to the main page](../README.md)

#### All requirements needed for the CLI will come packaged with this app.
This means that you won't need Node.JS or asar on your system.

<br/>

### ⚠️ HELP NEEDED: If you know how to properly sign an MSIX for free, please contact me

<br/>

## ⬇️ Installation
#### IMPORTANT: Do **NOT** delete or move the gui folder as it will require you to redo step 5 and everything after it

1. Install Visual Studio 2022 with ".NET Desktop Developement" and "UWP Developement"
2. Enable Windows Developer Mode
3. Clone the repository & open the solution (.sln) located in /gui in VS
4. Choose your CPU arch (e.g. x64) and "Release" in the toolbar located at the top
5. Click on the second green arrow in the toolbar (it's only partially filled)

**If this doesn't open the GUI, do the following:**

5. Open Powershell and cd into the following directory: ```/gui/WMPU-GUI/bin```
6. Go into the folder with your selected CPU arch (e.g. ```x64/```)
7. Go into the ```Release``` folder and afterwards into the folder inside there (it should be named something like ```net6.0-windows10.0.19041.0```)
8. Run ```Add-AppxPackage -Register .\AppxManifest.xml```
9. WeMod will now appear in your start menu and programs list


<br/>
