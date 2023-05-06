use crate::Player;
use yew::prelude::*;

fn square_with_hole_svg(sidelength: f32, radius: f32) -> Html {
    //TODO make private after testing
    let half_sidelength = sidelength / 2.;
    let diff = half_sidelength - radius;
    html! {

        <g width={sidelength.to_string()} height={sidelength.to_string()}>
            <path d={format!("M {half_sidelength} {half_sidelength}
                     m 0 -{half_sidelength}
                     l {half_sidelength} 0
                     l 0 {sidelength}
                     l -{sidelength} 0
                     l 0 -{sidelength}
                     l {half_sidelength} 0
                     Z
                     m 0 {diff}
                     a {radius} {radius} 0 1 1 -1 0
                     Z
                    ")}
                    fill-rule="evenodd"
                    class="block"/>
        </g>
    }
}

fn chip(player: &Player, winning: bool) -> Html {
    let (mut fill_color, rim_color) = match *player {
        Player::One => ("var(--red)", "var(--darkred)"),
        Player::Two => ("var(--blue)", "var(--darkblue)"),
    };

    let stroke_width;
    if winning {
        // rim_color = "#00A000";
        fill_color = "#2db020";
        stroke_width = "16%";
    } else {
        stroke_width = "10%";
    }

    html! {
        <g>
            <circle cx="50" cy="50" r="32.5" fill={fill_color} stroke={rim_color}
            stroke-width={stroke_width} />
        </g>
    }
}

#[derive(PartialEq, Properties)]
pub struct CellProps {
    pub status: Option<Player>,
    pub winning: bool,
}

#[function_component(Cell)]
pub fn cell(props: &CellProps) -> Html {
    // let chip_html = ;
    html! {
        <div class="cell">
            <svg height="100" width="100">
                <circle cx=50 cy=50 r=40 fill="var(--background-color)"/>
                {
                    if let Some(player) = &props.status {
                    chip(&player, props.winning)
                    } else {
                        html!{}
                    }
                }
                {square_with_hole_svg(100., 32.5)}
                // <rect width="100%" height="100%" style="fill:rgb(0,0,255)" />
            </svg>
        </div>
    }
}
