use super::api::TwitchClip;

pub fn has_duplicates(clips: Vec<TwitchClip>) -> bool {
    let mut clip_ids = Vec::new();
    for clip in clips {
        if clip_ids.contains(&clip.id) {
            return true;
        }
        clip_ids.push(clip.id);
    }
    false
}