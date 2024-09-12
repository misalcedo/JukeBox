use crate::spotify::models::{Album, DeviceList, Playlist, StartPlaybackRequest, Track};
use crate::token;
use reqwest::Result;
use url::Url;

pub mod models;

pub struct Uri {
    pub category: String,
    pub id: String,
}

impl<'a> TryFrom<&'a str> for Uri {
    type Error = &'a str;

    fn try_from(value: &'a str) -> std::result::Result<Self, Self::Error> {
        let Some(("spotify", parts)) = value.split_once(":") else {
            return Err(value);
        };
        let (category, id) = parts.split_once(":").ok_or(value)?;

        Ok(Uri {
            category: category.to_string(),
            id: id.to_string(),
        })
    }
}

impl TryFrom<Url> for Uri {
    type Error = Url;

    fn try_from(value: Url) -> std::result::Result<Self, Self::Error> {
        let mut path = value.path_segments().into_iter().flatten();
        match (value.scheme(), path.next(), path.next()) {
            ("spotify", None, None) => match Self::try_from(value.as_str()) {
                Ok(uri) => Ok(uri),
                Err(_) => Err(value),
            },
            ("https", Some(category), Some(id)) => Ok(Uri {
                category: category.to_string(),
                id: id.to_string(),
            }),
            _ => Err(value),
        }
    }
}

pub struct Client {
    oauth: token::Client,
    http: reqwest::blocking::Client,
}

impl Client {
    pub fn new(oauth: token::Client) -> Client {
        let http = reqwest::blocking::Client::new();
        Client { oauth, http }
    }

    pub fn get_available_devices(&mut self) -> Result<DeviceList> {
        self.http
            .get("https://api.spotify.com/v1/me/player/devices")
            .header("Authorization", self.oauth.authorization())
            .send()?
            .error_for_status()?
            .json()
    }

    pub fn play(
        &mut self,
        device_id: Option<String>,
        request: &StartPlaybackRequest,
    ) -> Result<()> {
        self.http
            .put("https://api.spotify.com/v1/me/player/play")
            .query(&[("device_id", device_id)])
            .header("Authorization", self.oauth.authorization())
            .json(request)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn shuffle(&mut self, state: bool) -> Result<()> {
        self.http
            .put("https://api.spotify.com/v1/me/player/shuffle")
            .query(&[("state", state)])
            .header("Authorization", self.oauth.authorization())
            .body("")
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn get_track(&mut self, id: &str) -> Result<Track> {
        self.http
            .put(format!("https://api.spotify.com/v1/tracks/{}", id))
            .header("Authorization", self.oauth.authorization())
            .body("")
            .send()?
            .error_for_status()?
            .json()
    }

    pub fn get_album(&mut self, id: &str) -> Result<Album> {
        self.http
            .put(format!("https://api.spotify.com/v1/albums/{}", id))
            .header("Authorization", self.oauth.authorization())
            .body("")
            .send()?
            .error_for_status()?
            .json()
    }

    pub fn get_playlist(&mut self, id: &str) -> Result<Playlist> {
        self.http
            .put(format!("https://api.spotify.com/v1/playlists/{}", id))
            .header("Authorization", self.oauth.authorization())
            .body("")
            .send()?
            .error_for_status()?
            .json()
    }
}

pub fn normalize_uri(uri: &Url) -> Option<String> {
    let mut path = uri.path_segments().into_iter().flatten();
    match (uri.scheme(), path.next(), path.next()) {
        ("spotify", None, None) => Some(uri.to_string()),
        ("https", Some(category), Some(id)) => Some(format!("spotify:{category}:{id}")),
        _ => None,
    }
}

pub fn uri_parts(uri: &str) -> Option<(&str, &str)> {
    let (_, parts) = uri.split_once(":")?;
    parts.split_once(":")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_track() {
        let url =
            Url::parse("https://open.spotify.com/track/6b2HYgqcK9mvktt4GxAu72?si=b71085cf9270496b")
                .unwrap();
        assert_eq!(
            normalize_uri(&url),
            Some("spotify:track:6b2HYgqcK9mvktt4GxAu72".to_string())
        );
    }

    #[test]
    fn normalize_track_no_op() {
        let url = Url::parse("spotify:track:6b2HYgqcK9mvktt4GxAu72").unwrap();
        assert_eq!(
            normalize_uri(&url),
            Some("spotify:track:6b2HYgqcK9mvktt4GxAu72".to_string())
        );
    }

    #[test]
    fn normalize_album() {
        let url = Url::parse(
            "https://open.spotify.com/album/2gSDW1mnuKSRLRa7pgTV4f?si=DBQxmqMLQXaezZ0GooOwjg",
        )
            .unwrap();
        assert_eq!(
            normalize_uri(&url),
            Some("spotify:album:2gSDW1mnuKSRLRa7pgTV4f".to_string())
        );
    }

    #[test]
    fn normalize_album_no_op() {
        let url = Url::parse("spotify:album:2gSDW1mnuKSRLRa7pgTV4f").unwrap();
        assert_eq!(
            normalize_uri(&url),
            Some("spotify:album:2gSDW1mnuKSRLRa7pgTV4f".to_string())
        );
    }

    #[test]
    fn normalize_playlist() {
        let url = Url::parse(
            "https://open.spotify.com/playlist/6sn3Heyme3WqK01uTNwoIp?si=c2f89da801b149d2",
        )
            .unwrap();
        assert_eq!(
            normalize_uri(&url),
            Some("spotify:playlist:6sn3Heyme3WqK01uTNwoIp".to_string())
        );
    }

    #[test]
    fn normalize_playlist_no_op() {
        let url = Url::parse("spotify:playlist:6sn3Heyme3WqK01uTNwoIp").unwrap();
        assert_eq!(
            normalize_uri(&url),
            Some("spotify:playlist:6sn3Heyme3WqK01uTNwoIp".to_string())
        );
    }

    #[test]
    fn normalize_http() {
        let url = Url::parse(
            "http://open.spotify.com/playlist/6sn3Heyme3WqK01uTNwoIp?si=c2f89da801b149d2",
        )
            .unwrap();
        assert_eq!(normalize_uri(&url), None);
    }

    #[test]
    fn normalize_no_id() {
        let url = Url::parse("https://open.spotify.com/playlist?si=c2f89da801b149d2").unwrap();
        assert_eq!(normalize_uri(&url), None);
    }

    #[test]
    fn split() {
        assert_eq!(
            uri_parts("spotify:playlist:6sn3Heyme3WqK01uTNwoIp"),
            Some(("playlist", "6sn3Heyme3WqK01uTNwoIp"))
        );
    }

    #[test]
    fn split_bad() {
        assert_eq!(uri_parts("spotify:6sn3Heyme3WqK01uTNwoIp"), None);
    }
}
