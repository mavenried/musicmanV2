#[cfg(test)]
mod tests {
    use musicman_protocol::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use uuid::Uuid;

    // Mock SongMeta for testing
    fn create_test_song(title: &str, artists: Vec<&str>, path: &str) -> (Uuid, SongMeta) {
        let path_buf = PathBuf::from(path);
        let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, path.as_bytes());
        let meta = SongMeta {
            id,
            title: title.to_string(),
            artists: artists.iter().map(|s| s.to_string()).collect(),
            duration: 180,
            path: path_buf,
        };
        (id, meta)
    }

    fn create_test_index() -> HashMap<Uuid, SongMeta> {
        let mut index = HashMap::new();

        let songs = vec![
            (
                "Bohemian Rhapsody",
                vec!["Queen"],
                "/music/queen/bohemian.mp3",
            ),
            (
                "Stairway to Heaven",
                vec!["Led Zeppelin"],
                "/music/led/stairway.flac",
            ),
            (
                "Hotel California",
                vec!["Eagles"],
                "/music/eagles/hotel.mp3",
            ),
            ("Imagine", vec!["John Lennon"], "/music/lennon/imagine.wav"),
            (
                "Smells Like Teen Spirit",
                vec!["Nirvana"],
                "/music/nirvana/smells.mp3",
            ),
            (
                "Collaboration",
                vec!["Artist A", "Artist B"],
                "/music/collab.mp3",
            ),
        ];

        for (title, artists, path) in songs {
            let (id, meta) = create_test_song(title, artists, path);
            index.insert(id, meta);
        }

        index
    }

    // Tests for handle_search
    mod search_tests {
        use super::*;
        use crate::handlers::handle_search;

        #[tokio::test]
        async fn test_search_by_title_exact_match() {
            let index = create_test_index();
            let results = handle_search(SearchType::ByTitle("Imagine".to_string()), &index).await;

            assert_eq!(results.len(), 1);
        }

        #[tokio::test]
        async fn test_search_by_title_partial_match() {
            let index = create_test_index();
            let results = handle_search(SearchType::ByTitle("spirit".to_string()), &index).await;

            assert_eq!(results.len(), 1);
        }

        #[tokio::test]
        async fn test_search_by_title_case_insensitive() {
            let index = create_test_index();
            let results = handle_search(SearchType::ByTitle("IMAGINE".to_string()), &index).await;

            assert_eq!(results.len(), 1);
        }

        #[tokio::test]
        async fn test_search_by_title_multiple_results() {
            let index = create_test_index();
            let results = handle_search(SearchType::ByTitle("a".to_string()), &index).await;

            // Should match multiple songs containing "a"
            assert!(results.len() > 1);
        }

        #[tokio::test]
        async fn test_search_by_title_no_results() {
            let index = create_test_index();
            let results =
                handle_search(SearchType::ByTitle("xyz123notfound".to_string()), &index).await;

            assert_eq!(results.len(), 0);
        }

        #[tokio::test]
        async fn test_search_by_artist_exact_match() {
            let index = create_test_index();
            let results = handle_search(SearchType::ByArtist("Queen".to_string()), &index).await;

            assert_eq!(results.len(), 1);
        }

        #[tokio::test]
        async fn test_search_by_artist_partial_match() {
            let index = create_test_index();
            let results = handle_search(SearchType::ByArtist("led".to_string()), &index).await;

            assert_eq!(results.len(), 1);
        }

        #[tokio::test]
        async fn test_search_by_artist_case_insensitive() {
            let index = create_test_index();
            let results = handle_search(SearchType::ByArtist("NIRVANA".to_string()), &index).await;

            assert_eq!(results.len(), 1);
        }

        #[tokio::test]
        async fn test_search_by_artist_multiple_artists_same_song() {
            let index = create_test_index();
            let results = handle_search(SearchType::ByArtist("Artist A".to_string()), &index).await;

            assert_eq!(results.len(), 1);
        }

        #[tokio::test]
        async fn test_search_empty_query() {
            let index = create_test_index();
            let results = handle_search(SearchType::ByTitle("".to_string()), &index).await;

            // Empty query should match all songs (all contain empty string)
            assert_eq!(results.len(), index.len());
        }

        #[tokio::test]
        async fn test_search_on_empty_index() {
            let index = HashMap::new();
            let results = handle_search(SearchType::ByTitle("test".to_string()), &index).await;

            assert_eq!(results.len(), 0);
        }
    }

    // Tests for UUID generation
    mod uuid_tests {
        use super::*;

        #[test]
        fn test_uuid_deterministic() {
            let path1 = "/music/test.mp3";
            let path2 = "/music/test.mp3";

            let id1 = Uuid::new_v5(&Uuid::NAMESPACE_URL, path1.as_bytes());
            let id2 = Uuid::new_v5(&Uuid::NAMESPACE_URL, path2.as_bytes());

            assert_eq!(id1, id2);
        }

        #[test]
        fn test_uuid_different_paths() {
            let path1 = "/music/song1.mp3";
            let path2 = "/music/song2.mp3";

            let id1 = Uuid::new_v5(&Uuid::NAMESPACE_URL, path1.as_bytes());
            let id2 = Uuid::new_v5(&Uuid::NAMESPACE_URL, path2.as_bytes());

            assert_ne!(id1, id2);
        }

        #[test]
        fn test_uuid_case_sensitive() {
            let path1 = "/music/Test.mp3";
            let path2 = "/music/test.mp3";

            let id1 = Uuid::new_v5(&Uuid::NAMESPACE_URL, path1.as_bytes());
            let id2 = Uuid::new_v5(&Uuid::NAMESPACE_URL, path2.as_bytes());

            // Paths are case-sensitive, so UUIDs should differ
            assert_ne!(id1, id2);
        }
    }

    // Tests for helper functions
    mod helper_tests {
        use super::*;
        use crate::helpers::get_track_meta;

        #[tokio::test]
        async fn test_get_track_meta_found() {
            let index = create_test_index();
            let (id, _) = create_test_song("Test", vec!["Artist"], "/test.mp3");
            let index_with_test = {
                let mut idx = index;
                let (id, meta) = create_test_song("Test", vec!["Artist"], "/test.mp3");
                idx.insert(id, meta);
                idx
            };

            let result = get_track_meta(&id, &index_with_test).await.unwrap();
            assert!(result.is_some());
            assert_eq!(result.unwrap().title, "Test");
        }

        #[tokio::test]
        async fn test_get_track_meta_not_found() {
            let index = create_test_index();
            let random_id = Uuid::new_v4();

            let result = get_track_meta(&random_id, &index).await.unwrap();
            assert!(result.is_none());
        }
    }

    // Edge case tests
    mod edge_cases {
        use super::*;
        use crate::handlers::handle_search;

        #[tokio::test]
        async fn test_search_with_special_characters() {
            let mut index = HashMap::new();
            let (id, meta) = create_test_song("Song (feat. Artist)", vec!["Test"], "/test.mp3");
            index.insert(id, meta);

            let results = handle_search(SearchType::ByTitle("feat".to_string()), &index).await;
            assert_eq!(results.len(), 1);
        }

        #[tokio::test]
        async fn test_search_with_numbers() {
            let mut index = HashMap::new();
            let (id, meta) = create_test_song("Track 01", vec!["Artist"], "/test.mp3");
            index.insert(id, meta);

            let results = handle_search(SearchType::ByTitle("01".to_string()), &index).await;
            assert_eq!(results.len(), 1);
        }

        #[tokio::test]
        async fn test_artist_split_by_slash() {
            let mut index = HashMap::new();
            let (id, meta) =
                create_test_song("Collab Song", vec!["Artist1", "Artist2"], "/test.mp3");
            index.insert(id, meta);

            // Search for first artist
            let results1 = handle_search(SearchType::ByArtist("Artist1".to_string()), &index).await;
            assert_eq!(results1.len(), 1);

            // Search for second artist
            let results2 = handle_search(SearchType::ByArtist("Artist2".to_string()), &index).await;
            assert_eq!(results2.len(), 1);
        }

        #[tokio::test]
        async fn test_duplicate_results_not_added() {
            let mut index = HashMap::new();
            let (id, meta) = create_test_song("Test Test", vec!["Artist"], "/test.mp3");
            index.insert(id, meta);

            // Even though "test" appears twice, should only get one result
            let results = handle_search(SearchType::ByTitle("test".to_string()), &index).await;
            assert_eq!(results.len(), 1);
        }
    }
}
