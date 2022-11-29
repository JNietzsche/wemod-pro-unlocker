using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Windows.Storage;

namespace WMPU_GUI.Utils
{
    public class UnlockManager
    {

        private StorageFolder wmpuDir = ApplicationData.Current.LocalFolder;
        private string wemodDir;
        private string wemodVer;

        public UnlockManager(string wemodDir, string wemodVer)
        {
            this.wemodDir = wemodDir;
            this.wemodVer = wemodVer;
        }

        private FileInfo getWMPUExe()
        {
            var files = new DirectoryInfo(wmpuDir.Path)
                .EnumerateFiles()
                .Where(file =>
                    file.Name.StartsWith("wemod-pro-unlocker-v")
                    && file.Name.EndsWith(".exe")
                )
                .ToList();

            files.Sort();

            return files.First();
        }

        public void KillWeMod()
        {
            var wemodProcesses = Process.GetProcesses().
                Where(pr => pr.ProcessName.ToLower() == "wemod");

            foreach (var process in wemodProcesses)
            {
                process.Kill();
            }
        }

        public Task Unlock()
        {
            using (var proc = new Process())
            {
                var args = $"--asar \"{Windows.ApplicationModel.Package.Current.InstalledPath + "/Assets"}\" --asar-bin asar.exe";

                if(wemodDir != null)
                {
                    args += $" --wemod-dir {wemodDir}";
                }

                if(wemodVer != null)
                {
                    args += $" --wemod-version {wemodVer}";
                }

                proc.StartInfo.FileName = getWMPUExe().FullName;
                proc.StartInfo.RedirectStandardInput = true;
                proc.StartInfo.RedirectStandardOutput = true;
                proc.StartInfo.CreateNoWindow = true;
                proc.StartInfo.UseShellExecute = false;
                proc.StartInfo.Arguments = args;
                proc.Start();

                while (!proc.HasExited)
                {
                    // var output = proc.StandardOutput.ReadLine();
                }

                KillWeMod();
            }

            return Task.CompletedTask;
        }
    }
}
