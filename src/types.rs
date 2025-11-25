use serde::{Deserialize, Serialize};

/// NHK Radio configuration from config_web.xml
#[derive(Debug, Deserialize)]
#[serde(rename = "radiru_config")]
#[allow(dead_code)]
pub struct RadiruConfig {
    pub info: String,
    pub stream_url: StreamUrl,
    pub url_program_noa: String,
    pub url_program_day: String,
    pub url_program_detail: String,
    pub radiru_twitter_timeline: String,
}

#[derive(Debug, Deserialize)]
pub struct StreamUrl {
    #[serde(rename = "data", default)]
    pub data: Vec<StreamData>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StreamData {
    pub areajp: String,
    pub area: String,
    pub apikey: String,
    pub areakey: String,
    pub r1hls: String,
    pub r2hls: String,
    pub fmhls: String,
}

/// Program information root
#[derive(Debug, Deserialize, Serialize)]
pub struct Root {
    pub r1: Channel,
    pub r2: Channel,
    pub r3: Channel,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Channel {
    #[serde(default)]
    pub previous: Option<BroadcastEvent>,
    #[serde(default)]
    pub present: Option<BroadcastEvent>,
    #[serde(default)]
    pub following: Option<BroadcastEvent>,
    #[serde(rename = "publishedOn", default)]
    pub published_on: Option<BroadcastService>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BroadcastEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
    pub location: Location,
    #[serde(rename = "identifierGroup")]
    pub identifier_group: IdentifierGroup,
    pub misc: Misc,
    pub url: String,
    #[serde(default)]
    pub about: Option<About>,
    #[serde(rename = "eyecatchList", default)]
    pub eyecatch_list: Vec<Images>,
    #[serde(rename = "additionalProperty", default)]
    pub additional_property: Option<AdditionalProperty>,
    #[serde(default)]
    pub audio: Vec<Audio>,
    #[serde(rename = "isLiveBroadcast")]
    pub is_live_broadcast: bool,
    #[serde(rename = "detailedDescription")]
    pub detailed_description: DetailedDescription,
    pub duration: String,
    #[serde(rename = "posterframeList", default)]
    pub posterframe_list: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Location {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct IdentifierGroup {
    #[serde(rename = "broadcastEventId", default)]
    pub broadcast_event_id: String,
    #[serde(rename = "radioEpisodeId", default)]
    pub radio_episode_id: String,
    #[serde(rename = "radioEpisodeName", default)]
    pub radio_episode_name: String,
    #[serde(rename = "radioSeriesId", default)]
    pub radio_series_id: String,
    #[serde(rename = "radioSeriesName", default)]
    pub radio_series_name: String,
    #[serde(rename = "serviceId", default)]
    pub service_id: String,
    #[serde(rename = "areaId", default)]
    pub area_id: String,
    #[serde(rename = "stationId", default)]
    pub station_id: String,
    #[serde(default)]
    pub date: String,
    #[serde(rename = "eventId", default)]
    pub event_id: String,
    #[serde(default)]
    pub genre: Vec<Genre>,
    #[serde(rename = "siteId", default)]
    pub site_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Genre {
    pub id: String,
    pub name1: String,
    pub name2: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Misc {
    #[serde(rename = "displayVideoMode")]
    pub display_video_mode: String,
    #[serde(rename = "displayVideoRange")]
    pub display_video_range: String,
    #[serde(rename = "displayAudioMode", default)]
    pub display_audio_mode: Vec<String>,
    #[serde(rename = "audioMode", default)]
    pub audio_mode: Vec<String>,
    #[serde(rename = "supportCaption")]
    pub support_caption: bool,
    #[serde(rename = "supportSign")]
    pub support_sign: bool,
    #[serde(rename = "supportHybridcast")]
    pub support_hybridcast: bool,
    #[serde(rename = "supportDataBroadcast")]
    pub support_data_broadcast: bool,
    #[serde(rename = "isInteractive")]
    pub is_interactive: bool,
    #[serde(rename = "isChangeable")]
    pub is_changeable: bool,
    #[serde(rename = "releaseLevel")]
    pub release_level: String,
    #[serde(rename = "programType")]
    pub program_type: String,
    pub coverage: String,
    #[serde(rename = "actList", default)]
    pub act_list: Vec<Act>,
    #[serde(rename = "musicList", default)]
    pub music_list: Vec<Music>,
    #[serde(rename = "eventShareStatus")]
    pub event_share_status: String,
    #[serde(rename = "playControlSimul")]
    pub play_control_simul: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Act {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "nameRuby", default)]
    pub name_ruby: String,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Music {
    pub name: String,
    #[serde(rename = "nameruby")]
    pub name_ruby: String,
    pub lyricist: String,
    pub composer: String,
    pub arranger: String,
    pub location: String,
    pub provider: String,
    pub label: String,
    pub duration: String,
    pub code: String,
    #[serde(rename = "byArtist", default)]
    pub by_artist: Vec<Artist>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Artist {
    pub name: String,
    pub role: String,
    pub part: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct About {
    pub id: String,
    pub name: String,
    #[serde(rename = "detailedEpisodeNameRuby", default)]
    pub detailed_episode_name_ruby: Option<String>,
    #[serde(rename = "identifierGroup")]
    pub identifier_group: AboutIdentifierGroup,
    #[serde(default)]
    pub keyword: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "partOfSeries")]
    pub part_of_series: PartOfSeries,
    #[serde(default)]
    pub eyecatch: Option<Images>,
    #[serde(rename = "eyecatchList", default)]
    pub eyecatch_list: Vec<Images>,
    pub url: String,
    #[serde(default)]
    pub canonical: Option<String>,
    #[serde(rename = "additionalProperty", default)]
    pub additional_property: Option<AdditionalProperty>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AboutIdentifierGroup {
    #[serde(rename = "radioEpisodeId", default)]
    pub radio_episode_id: String,
    #[serde(rename = "radioSeriesId", default)]
    pub radio_series_id: String,
    #[serde(rename = "radioEpisodeName", default)]
    pub radio_episode_name: String,
    #[serde(rename = "radioSeriesName", default)]
    pub radio_series_name: String,
    #[serde(default)]
    pub hashtag: Vec<String>,
    #[serde(rename = "siteId", default)]
    pub site_id: Option<String>,
    #[serde(rename = "aliasId", default)]
    pub alias_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PartOfSeries {
    pub id: String,
    pub name: String,
    #[serde(rename = "detailedSeriesNameRuby", default)]
    pub detailed_series_name_ruby: Option<String>,
    #[serde(rename = "identifierGroup")]
    pub identifier_group: SeriesIdentifierGroup,
    #[serde(default)]
    pub keyword: Vec<String>,
    #[serde(rename = "detailedSynonym", default)]
    pub detailed_synonym: Vec<String>,
    #[serde(rename = "sameAs", default)]
    pub same_as: Vec<SameAs>,
    #[serde(default)]
    pub canonical: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "detailedCatch", default)]
    pub detailed_catch: Option<String>,
    pub logo: Images,
    pub eyecatch: Images,
    pub hero: Images,
    #[serde(default)]
    pub style: Style,
    #[serde(rename = "additionalProperty", default)]
    pub additional_property: AdditionalProperty,
    pub url: String,
    #[serde(rename = "itemUrl")]
    pub item_url: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SeriesIdentifierGroup {
    #[serde(rename = "radioSeriesId", default)]
    pub radio_series_id: String,
    #[serde(rename = "radioSeriesPlaylistId", default)]
    pub radio_series_playlist_id: String,
    #[serde(rename = "radioSeriesUId", default)]
    pub radio_series_uid: String,
    #[serde(rename = "radioSeriesName", default)]
    pub radio_series_name: String,
    #[serde(default)]
    pub hashtag: Vec<String>,
    #[serde(rename = "siteId", default)]
    pub site_id: Option<String>,
    #[serde(rename = "aliasId", default)]
    pub alias_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SameAs {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Image {
    pub url: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Images {
    pub large: Option<Image>,
    pub main: Option<Image>,
    pub medium: Option<Image>,
    pub small: Option<Image>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Style {
    #[serde(rename = "textLight", default)]
    pub text_light: String,
    #[serde(rename = "textDark", default)]
    pub text_dark: String,
    #[serde(rename = "linkLight", default)]
    pub link_light: String,
    #[serde(rename = "linkDark", default)]
    pub link_dark: String,
    #[serde(rename = "primaryLight", default)]
    pub primary_light: String,
    #[serde(rename = "primaryDark", default)]
    pub primary_dark: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AdditionalProperty {
    #[serde(rename = "publishLevel", default)]
    pub publish_level: String,
    #[serde(rename = "layoutPattern", default)]
    pub layout_pattern: String,
    #[serde(rename = "episodeOrderBy", default)]
    pub episode_order_by: String,
    #[serde(rename = "availableOnPlus", default)]
    pub available_on_plus: bool,
    #[serde(rename = "enableVariablePlayBackSpeedControl", default)]
    pub enable_variable_playback_speed_control: bool,
    #[serde(default)]
    pub optional: Vec<String>,
    #[serde(rename = "seriesPackStatus", default)]
    pub series_pack_status: String,
    #[serde(rename = "supportMedia", default)]
    pub support_media: Vec<String>,
    #[serde(rename = "supportMusicList", default)]
    pub support_music_list: bool,
    #[serde(rename = "supportPlusEmbed", default)]
    pub support_plus_embed: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Audio {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    #[serde(rename = "identifierGroup")]
    pub identifier_group: AudioIdentifierGroup,
    #[serde(rename = "detailedContentStatus")]
    pub detailed_content_status: DetailedContentStatus,
    #[serde(rename = "detailedContent", default)]
    pub detailed_content: Vec<DetailedContent>,
    pub duration: String,
    #[serde(default)]
    pub publication: Vec<Publication>,
    #[serde(rename = "hasPart", default)]
    pub has_part: Vec<String>,
    pub expires: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AudioIdentifierGroup {
    #[serde(rename = "environmentId")]
    pub environment_id: String,
    #[serde(rename = "broadcastEventId")]
    pub broadcast_event_id: String,
    #[serde(rename = "streamType")]
    pub stream_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DetailedContentStatus {
    #[serde(rename = "environmentId")]
    pub environment_id: String,
    #[serde(rename = "streamType")]
    pub stream_type: String,
    #[serde(rename = "contentStatus")]
    pub content_status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DetailedContent {
    pub name: String,
    #[serde(rename = "contentUrl")]
    pub content_url: String,
    #[serde(rename = "encodingFormat", default)]
    pub encoding_format: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Publication {
    pub id: String,
    pub url: String,
    #[serde(rename = "isLiveBroadcast")]
    pub is_live_broadcast: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DetailedDescription {
    #[serde(default)]
    pub epg40: String,
    #[serde(default)]
    pub epg80: String,
    #[serde(rename = "epgInformation", default)]
    pub epg_information: String,
    #[serde(default)]
    pub epg200: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BroadcastService {
    #[serde(rename = "type")]
    pub service_type: String,
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(rename = "broadcastDisplayName")]
    pub broadcast_display_name: String,
    #[serde(rename = "videoFormat", default)]
    pub video_format: Vec<String>,
    #[serde(rename = "encodingFormat", default)]
    pub encoding_format: Vec<String>,
    #[serde(rename = "identifierGroup")]
    pub identifier_group: ServiceIdentifierGroup,
    pub logo: Images,
    pub eyecatch: Images,
    pub hero: Images,
    #[serde(rename = "badge9x4")]
    pub badge_9x4: Images,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ServiceIdentifierGroup {
    #[serde(rename = "serviceId", default)]
    pub service_id: String,
    #[serde(rename = "serviceName", default)]
    pub service_name: String,
    #[serde(rename = "areaId", default)]
    pub area_id: String,
    #[serde(rename = "areaName", default)]
    pub area_name: String,
    #[serde(rename = "channelId", default)]
    pub channel_id: Option<String>,
    #[serde(rename = "channelKey", default)]
    pub channel_key: Option<String>,
    #[serde(rename = "channelAreaName", default)]
    pub channel_area_name: String,
    #[serde(rename = "channelStationName", default)]
    pub channel_station_name: String,
    #[serde(rename = "shortenedName", default)]
    pub shortened_name: String,
    #[serde(rename = "shortenedDisplayName", default)]
    pub shortened_display_name: String,
    #[serde(rename = "multiChannelDisplayName", default)]
    pub multi_channel_display_name: Option<String>,
}

/// Segment information from M3U8 playlist
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Segment {
    pub url: String,
    pub key_url: Option<String>,
    pub iv: Option<String>,
    pub seq_no: u64,
    pub duration: f64,
}
