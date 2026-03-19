#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn contains(self, px: u16, py: u16) -> bool {
        px >= self.x
            && px < self.x.saturating_add(self.width)
            && py >= self.y
            && py < self.y.saturating_add(self.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionLayout {
    pub header: Rect,
    pub hints: Rect,
    pub threads: Rect,
    pub chat: Rect,
    pub mission: Rect,
    pub composer_meta: Rect,
    pub composer_input: Rect,
    pub status: Rect,
}

pub fn mission_layout(width: u16, height: u16) -> MissionLayout {
    let width = width.max(80);
    let height = height.max(24);

    let body_y = 2;
    let body_h = height.saturating_sub(6);

    let min_left = 22;
    let min_chat = 28;
    let min_mission = 22;
    let separators = 6;
    let available = width.saturating_sub(separators);

    let mut left_w = ((available as f32) * 0.24) as u16;
    left_w = left_w.clamp(min_left, 36);

    let mut mission_w = ((available as f32) * 0.26) as u16;
    mission_w = mission_w.clamp(min_mission, 44);

    if left_w + mission_w + min_chat > available {
        let mut overflow = left_w + mission_w + min_chat - available;
        let mission_reduce = overflow.min(mission_w.saturating_sub(min_mission));
        mission_w = mission_w.saturating_sub(mission_reduce);
        overflow = overflow.saturating_sub(mission_reduce);

        let left_reduce = overflow.min(left_w.saturating_sub(min_left));
        left_w = left_w.saturating_sub(left_reduce);
    }

    let chat_w = available.saturating_sub(left_w + mission_w).max(min_chat);
    let chat_x = left_w + 3;
    let mission_x = chat_x + chat_w + 3;

    MissionLayout {
        header: Rect {
            x: 0,
            y: 0,
            width,
            height: 1,
        },
        hints: Rect {
            x: 0,
            y: 1,
            width,
            height: 1,
        },
        threads: Rect {
            x: 0,
            y: body_y,
            width: left_w,
            height: body_h,
        },
        chat: Rect {
            x: chat_x,
            y: body_y,
            width: chat_w,
            height: body_h,
        },
        mission: Rect {
            x: mission_x,
            y: body_y,
            width: mission_w,
            height: body_h,
        },
        composer_meta: Rect {
            x: 0,
            y: height.saturating_sub(3),
            width,
            height: 1,
        },
        composer_input: Rect {
            x: 0,
            y: height.saturating_sub(2),
            width,
            height: 1,
        },
        status: Rect {
            x: 0,
            y: height.saturating_sub(1),
            width,
            height: 1,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{mission_layout, Rect};

    fn right(rect: Rect) -> u16 {
        rect.x.saturating_add(rect.width)
    }

    #[test]
    fn mission_layout_has_three_non_overlapping_body_panels() {
        let layout = mission_layout(160, 48);

        assert!(layout.threads.width > 0);
        assert!(layout.chat.width > 0);
        assert!(layout.mission.width > 0);

        assert_eq!(layout.chat.x, right(layout.threads) + 3);
        assert_eq!(layout.mission.x, right(layout.chat) + 3);
        assert_eq!(right(layout.mission), layout.header.width);
    }

    #[test]
    fn mission_layout_clamps_small_terminals_safely() {
        let layout = mission_layout(40, 10);

        // Layout uses internal minimums (80x24) to avoid collapsing panels.
        assert_eq!(layout.header.width, 80);
        assert_eq!(layout.status.y + 1, 24);

        assert!(layout.threads.width >= 22);
        assert!(layout.chat.width >= 28);
        assert!(layout.mission.width >= 22);
    }
}
