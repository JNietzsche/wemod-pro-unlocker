using Newtonsoft.Json;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using Windows.System;
using System.Globalization;
using Newtonsoft.Json.Converters;
using Microsoft.UI.Dispatching;

namespace WMPU_GUI.Utils
{
    public partial class GithubRelease
    {
        [JsonProperty("url")]
        public Uri Url { get; set; }

        [JsonProperty("assets_url")]
        public Uri AssetsUrl { get; set; }

        [JsonProperty("upload_url")]
        public string UploadUrl { get; set; }

        [JsonProperty("html_url")]
        public Uri HtmlUrl { get; set; }

        [JsonProperty("id")]
        public long Id { get; set; }

        [JsonProperty("author")]
        public Author Author { get; set; }

        [JsonProperty("node_id")]
        public string NodeId { get; set; }

        [JsonProperty("tag_name")]
        public string TagName { get; set; }

        [JsonProperty("target_commitish")]
        public string TargetCommitish { get; set; }

        [JsonProperty("name")]
        public string Name { get; set; }

        [JsonProperty("draft")]
        public bool Draft { get; set; }

        [JsonProperty("prerelease")]
        public bool Prerelease { get; set; }

        [JsonProperty("created_at")]
        public DateTimeOffset CreatedAt { get; set; }

        [JsonProperty("published_at")]
        public DateTimeOffset PublishedAt { get; set; }

        [JsonProperty("assets")]
        public Asset[] Assets { get; set; }

        [JsonProperty("tarball_url")]
        public Uri TarballUrl { get; set; }

        [JsonProperty("zipball_url")]
        public Uri ZipballUrl { get; set; }

        [JsonProperty("body")]
        public string Body { get; set; }
    }

    public partial class Asset
    {
        [JsonProperty("url")]
        public Uri Url { get; set; }

        [JsonProperty("id")]
        public long Id { get; set; }

        [JsonProperty("node_id")]
        public string NodeId { get; set; }

        [JsonProperty("name")]
        public string Name { get; set; }

        [JsonProperty("label")]
        public object Label { get; set; }

        [JsonProperty("uploader")]
        public Author Uploader { get; set; }

        [JsonProperty("content_type")]
        public string ContentType { get; set; }

        [JsonProperty("state")]
        public string State { get; set; }

        [JsonProperty("size")]
        public long Size { get; set; }

        [JsonProperty("download_count")]
        public long DownloadCount { get; set; }

        [JsonProperty("created_at")]
        public DateTimeOffset CreatedAt { get; set; }

        [JsonProperty("updated_at")]
        public DateTimeOffset UpdatedAt { get; set; }

        [JsonProperty("browser_download_url")]
        public Uri BrowserDownloadUrl { get; set; }
    }

    public partial class Author
    {
        [JsonProperty("login")]
        public string Login { get; set; }

        [JsonProperty("id")]
        public long Id { get; set; }

        [JsonProperty("node_id")]
        public string NodeId { get; set; }

        [JsonProperty("avatar_url")]
        public Uri AvatarUrl { get; set; }

        [JsonProperty("gravatar_id")]
        public string GravatarId { get; set; }

        [JsonProperty("url")]
        public Uri Url { get; set; }

        [JsonProperty("html_url")]
        public Uri HtmlUrl { get; set; }

        [JsonProperty("followers_url")]
        public Uri FollowersUrl { get; set; }

        [JsonProperty("following_url")]
        public string FollowingUrl { get; set; }

        [JsonProperty("gists_url")]
        public string GistsUrl { get; set; }

        [JsonProperty("starred_url")]
        public string StarredUrl { get; set; }

        [JsonProperty("subscriptions_url")]
        public Uri SubscriptionsUrl { get; set; }

        [JsonProperty("organizations_url")]
        public Uri OrganizationsUrl { get; set; }

        [JsonProperty("repos_url")]
        public Uri ReposUrl { get; set; }

        [JsonProperty("events_url")]
        public string EventsUrl { get; set; }

        [JsonProperty("received_events_url")]
        public Uri ReceivedEventsUrl { get; set; }

        [JsonProperty("type")]
        public string Type { get; set; }

        [JsonProperty("site_admin")]
        public bool SiteAdmin { get; set; }
    }

    public partial class GithubRelease
    {
        public static GithubRelease FromJson(string json) => JsonConvert.DeserializeObject<GithubRelease>(json, Utils.Converter.Settings);
    }

    public static class Serialize
    {
        public static string ToJson(this GithubRelease self) => JsonConvert.SerializeObject(self, Utils.Converter.Settings);
    }

    internal static class Converter
    {
        public static readonly JsonSerializerSettings Settings = new JsonSerializerSettings
        {
            MetadataPropertyHandling = MetadataPropertyHandling.Ignore,
            DateParseHandling = DateParseHandling.None,
            Converters = {
        new IsoDateTimeConverter { DateTimeStyles = DateTimeStyles.AssumeUniversal }
      },
        };
    }

    public class UpdateManager
    {
        public const string repo = "bennett-sh/wemod-pro-unlocker";
        public const string apiUrl = "https://api.github.com/";
        private HttpClient apiClient;
        private HttpClient downloadClient;
        Windows.Storage.StorageFolder localFolder = Windows.Storage.ApplicationData.Current.LocalFolder;

        public UpdateManager()
        {
            InitClient();
        }

        public void InitClient()
        {
            apiClient = new HttpClient
            {
                BaseAddress = new Uri(apiUrl)
            };
            apiClient.DefaultRequestHeaders.Add("User-Agent", "WeMod-Pro-Unlocker GUI");

            downloadClient = new HttpClient();
            downloadClient.DefaultRequestHeaders.Add("User-Agent", "WeMod-Pro-Unlocker GUI");
        }

        public async Task<GithubRelease?> CheckForWMPUUpdate()
        {
            var response = await apiClient.GetAsync($"/repos/{repo}/releases/latest");

            if (response.IsSuccessStatusCode && response.Content is object && response.Content.Headers.ContentType.MediaType == "application/json")
            {
                var contentStream = await response.Content.ReadAsStreamAsync();

                using var streamReader = new StreamReader(contentStream);
                using var jsonReader = new JsonTextReader(streamReader);

                Newtonsoft.Json.JsonSerializer serializer = new();

                try
                {
                    var result = serializer.Deserialize<GithubRelease>(jsonReader);
                    var tagName = result.TagName;

                    if (!File.Exists($"{localFolder.Path}\\wemod-pro-unlocker-{tagName}.exe"))
                    {
                        // Automatically install on first launch
                        if(new DirectoryInfo(localFolder.Path)
                            .EnumerateFiles()
                            .Select(file => file.Name.ToLower())
                            .ToList()
                            .Find(fn => fn.StartsWith("wemod-pro-unlocker-v") && fn.EndsWith(".exe")) == null)
                        {
                            await UpdateWMPU(result);
                            return null;
                        }

                        return result;
                    }

                }
                catch (JsonReaderException)
                {
                    Debug.WriteLine("Invalid JSON");
                }
            }

            return null;
        }

        public void RemoveWMPU()
        {
            new DirectoryInfo(localFolder.Path)
                .EnumerateFiles()
                .Where(file =>
                    file.Name.StartsWith("wemod-pro-unlocker-v")
                    && file.Name.EndsWith(".exe")
                )
                .ToList()
                .ForEach(file => file.Delete());
        }
        
        public async Task UpdateWMPU(GithubRelease release)
        {
            var response = await downloadClient.GetAsync(release.Assets.ToList().Find(asset => asset.Name.ToLower() == "wemod-pro-unlocker.exe").BrowserDownloadUrl);

            if(response.IsSuccessStatusCode)
            {
                // Remove old versions
                RemoveWMPU();

                var stream = await response.Content.ReadAsStreamAsync();
                FileInfo fileInfo = new($"{localFolder.Path}\\wemod-pro-unlocker-{release.TagName}.exe");
                using (var fileStream = fileInfo.OpenWrite())
                {
                    await stream.CopyToAsync(fileStream);
                }
            }
        }
    }
}
