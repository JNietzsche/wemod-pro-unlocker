// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License. See LICENSE in the project root for license information.

using Microsoft.UI.Composition.SystemBackdrops;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Navigation;
using System;
using WinRT;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices.WindowsRuntime;
using System.Threading.Tasks;
using Windows.Foundation;
using Windows.Foundation.Collections;
using WMPU_GUI.Utils;
using Windows.Storage.Pickers;
using Windows.Storage;

namespace WMPU_GUI
{
    /// <summary>
    /// An empty window that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class MainWindow : Window
    {
        Microsoft.UI.Dispatching.DispatcherQueue dispatcherQueue = Microsoft.UI.Dispatching.DispatcherQueue.GetForCurrentThread();
        public UpdateManager updateManager;
        public GithubRelease availableUpdate;
        MicaController m_backdropController;
        SystemBackdropConfiguration m_configurationSource;
        WindowsSystemDispatcherQueueHelper m_wsdqHelper;
        StorageFolder wemodDirectory;
        string weModVersion;

        public MainWindow()
        {
            this.InitializeComponent();
            updateManager = new UpdateManager();

            TrySetSystemBackdrop();

            IntPtr hWnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
            var windowId = Microsoft.UI.Win32Interop.GetWindowIdFromWindow(hWnd);
            var appWindow = Microsoft.UI.Windowing.AppWindow.GetFromWindowId(windowId);

            appWindow.Resize(new Windows.Graphics.SizeInt32 { Width = 762, Height = 578 });
            appWindow.SetIcon("Assets/appicon.ico");
            appWindow.Title = "WeMod Pro Unlocker GUI";

            Task.Run(async () => {
                var update = await updateManager.CheckForWMPUUpdate();

                if (update != null)
                {
                    this.availableUpdate = update;
                    dispatcherQueue.TryEnqueue(() =>
                    {
                        UpdateAvailableTip.IsOpen = true;
                    });
                }
            });

            OnWeModFolderUpdate();

        }


        public void OnWeModFolderUpdate()
        {
            wemodVersionCombo.Items.Clear();

            string dir;

            if(wemodDirectory == null)
            {
                dir = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData) + @"\WeMod";
            } else
            {
                dir = wemodDirectory.Path;
            }

            var versionDirs = new DirectoryInfo(dir)
                .EnumerateDirectories()
                .Where(dir => dir.Name.StartsWith("app-"))
                .ToList();

            wemodVersionCombo.SelectedIndex = versionDirs.Count - 1;

            versionDirs
                .ForEach(dir =>
            {
                var version = dir.Name.Substring(4);

                wemodVersionCombo.Items.Add(version);
            });
        }


        bool TrySetSystemBackdrop()
        {
            if (MicaController.IsSupported())
            {
                m_wsdqHelper = new WindowsSystemDispatcherQueueHelper();
                m_wsdqHelper.EnsureWindowsSystemDispatcherQueueController();

                // Create the policy object.
                m_configurationSource = new SystemBackdropConfiguration();
                this.Activated += Window_Activated;
                this.Closed += Window_Closed;
                ((FrameworkElement)this.Content).ActualThemeChanged += Window_ThemeChanged;

                // Initial configuration state.
                m_configurationSource.IsInputActive = true;
                SetConfigurationSourceTheme();

                m_backdropController = new MicaController()
                {
                    Kind = MicaKind.Base
                };

                // Enable the system backdrop.
                m_backdropController.AddSystemBackdropTarget(this.As<Microsoft.UI.Composition.ICompositionSupportsSystemBackdrop>());
                m_backdropController.SetSystemBackdropConfiguration(m_configurationSource);
                return true; // succeeded
            }

            return false; // Mica is not supported on this system
        }

        private void Window_Activated(object sender, WindowActivatedEventArgs args)
        {
            m_configurationSource.IsInputActive = args.WindowActivationState != WindowActivationState.Deactivated;
        }

        private void Window_Closed(object sender, WindowEventArgs args)
        {
            // Make sure any Mica/Acrylic controller is disposed
            // so it doesn't try to use this closed window.
            if (m_backdropController != null)
            {
                m_backdropController.Dispose();
                m_backdropController = null;
            }
            this.Activated -= Window_Activated;
            m_configurationSource = null;
        }

        private void Window_ThemeChanged(FrameworkElement sender, object args)
        {
            if (m_configurationSource != null)
            {
                SetConfigurationSourceTheme();
            }
        }

        private void SetConfigurationSourceTheme()
        {
            switch (((FrameworkElement)this.Content).ActualTheme)
            {
                case ElementTheme.Dark: m_configurationSource.Theme = SystemBackdropTheme.Dark; break;
                case ElementTheme.Light: m_configurationSource.Theme = SystemBackdropTheme.Light; break;
                case ElementTheme.Default: m_configurationSource.Theme = SystemBackdropTheme.Default; break;
            }
        }

        public void Alert(string title, string? content)
        {
            ttAlert.Title = title;
            ttAlert.Content = content;
            ttAlert.IsOpen = true;
        }

        private async void UpdateAvailableTip_ActionButtonClick(TeachingTip sender, object args)
        {
            if (this.availableUpdate != null)
            {
                await updateManager.UpdateWMPU(availableUpdate);
                availableUpdate = null;
                UpdateAvailableTip.IsOpen = false;
                Alert("Update installed", "The update has been installed.");
            }
        }

        private void unlockBtn_Click(object sender, RoutedEventArgs e)
        {
            string dir;

            if (wemodDirectory == null)
            {
                dir = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData) + @"\WeMod";
            }
            else
            {
                dir = wemodDirectory.Path;
            }

            unlockBtn.IsEnabled = false;
            unlockingRing.IsActive = true;
            unlockingRing.Visibility = Visibility.Visible;

            Task.Run(async () =>
            {
                await (new UnlockManager(
                    wemodDir: dir,
                    wemodVer: weModVersion
                ).Unlock());

                dispatcherQueue.TryEnqueue(() =>
                {
                    UnlockDone.IsOpen = true;
                    unlockBtn.IsEnabled = true;
                    unlockingRing.IsActive = false;
                    unlockingRing.Visibility = Visibility.Collapsed;
                });
            });
        }

        private void wemodVersionCombo_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            weModVersion = wemodVersionCombo.SelectedValue as string;
        }

        private async void wemodFolderBtn_Click(object sender, RoutedEventArgs e)
        {
            var FolderPicker = new FolderPicker
            {
                ViewMode = PickerViewMode.List,
                SuggestedStartLocation = PickerLocationId.ComputerFolder
            };
            var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
            WinRT.Interop.InitializeWithWindow.Initialize(FolderPicker, hwnd);
            wemodDirectory = await FolderPicker.PickSingleFolderAsync();
               
            OnWeModFolderUpdate();

            if(wemodDirectory != null)
            {
                wemodFolderText.Text = wemodDirectory.Path;
            } else
            {
                wemodFolderText.Text = "Default WeMod Folder";
            }
        }

        private void UnlockDone_ActionButtonClick(TeachingTip sender, object args)
        {
            string dir;

            if (wemodDirectory == null)
            {
                dir = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData) + @"\WeMod";
            }
            else
            {
                dir = wemodDirectory.Path;
            }

            var process = new Process();

            process.StartInfo.FileName = dir + @"\WeMod.exe";
            process.Start();

            Window.Current.Close();
        }
    }
}
