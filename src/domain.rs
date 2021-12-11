use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: String,
}

pub struct SubscriberName(String);

impl SubscriberName {
    pub fn inner(self) -> String {
        self.0
    }

    pub fn inner_ref(&self) -> &str {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl SubscriberName {
    pub fn parse(s: String) -> SubscriberName {
        // `.trim()` returns a view over the input `s` without trailing whitespace-like
        // characters
        // `.is_empty` checks if the view contains any character
        let is_empty_or_whitespace = s.trim().is_empty();
        // A grapheme is defined by the unicode standard as a "user-perceived"
        // character `å` is a single grapheme, but it's composed of two characters
        //
        // `graphemes` returns an iterator over the graphemes for input `s`
        // `true` specifies that we want to use the extended grapheme definition set,
        // the recommended one
        let is_too_long = s.graphemes(true).count() > 256;
        // Iterate over all characters in the input `s` to check if any of them matches one of
        // the characters in the forbidden array
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));
        // Return `false` if any of our conditions have been violated
        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            panic!("{} is not a valid subscriber name.", s)
        } else {
            Self(s)
        }
    }
}
