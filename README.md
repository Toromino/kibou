# Kibou

## What is Kibou?
Kibou is a lightweight federated social networking server based on open protocols. It is
written in Rust, utilizes Rocket as it's web framework and Diesel as it's database driver. The project's objective is to provide a highly customizable multi-protocol social networking server. Currently supported is the commonly used [ActivityPub](https://activitypub.rocks) protocol. 

Furthermore Kibou implements [Mastodon's REST API](https://docs.joinmastodon.org/api), this means that all applications for [Mastodon](https://joinmastodon.org) should also work with Kibou.

Kibou ships with it's own user interface called Raito-FE. In it's standard configuration it's completely based on static components and does not use any JavaScript. Although dynamic components (such as automatically refreshing timelines and dynamic routing) can optionally be enabled. A `minimal mode` can also be enabled in it's settings which reduces network traffic and only renders Raito-FE's core components.

![Kibou UI screenshot](https://git.cybre.club/attachments/ed7cacce-058e-47d3-8544-7584516a55d9)

## Try it out
[List of Kibou nodes](https://git.cybre.club/kibouproject/kibou/wiki/List-of-nodes)

## Federation with other software
Federation is known to work with [Pleroma](https://pleroma.social), [Misskey](https://joinmisskey.github.io) and [Mastodon](https://joinmastodon.org) which are also the main projects being tested against. But federation with other software should work as well.

## Get in touch
Join the IRC channel `#kibou` on freenode.net
