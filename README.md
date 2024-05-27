<div align="center">

[Installation](#installation) |
[URL Formats](#format) |
[Key Generation](#key) |
[Command Usage](#command)

# Shitpost

[![GitHub](https://img.shields.io/badge/Github-KaitlynEthylia%2Fshitpost-cec2fc?logo=github&style=for-the-badge)](https://github.com/KaitlynEthylia/shitpost)
![Overengineered](https://custom-icon-badges.demolab.com/badge/Over-Engineered-f25788?logo=heart&style=for-the-badge)
[![Crates.io](https://img.shields.io/crates/v/shitpost?color=%23f7b679&logo=rust&style=for-the-badge)](https://crates.io/crates/shitpost)

A compile-time Markov bot creator for the Fediverse.

</div>

<a id="installation"></a>

## Installation

Because the bot is generated at compile time, you need to pass it some
environment variables at build time in order for it to retrieve posts
to train on.

| Variable | Required | Description |
| -------- | -------- | ----------- |
| `$SHITPOST_IN` | `true` | The URL to retrieve posts from to train the bot. Training a bot on posts that aren't you're own is generally considered a 'dick move'. Don't be a dick. |
| `$SHITPOST_KEY` | `false` | A private key with `read` permissions. If this is set, the bot will be able to train off of non-public posts. The key is only used whilst training the bot and is stored nowhere in the compiled binary. |
| `$SHITPOST_OUT` | `false` | The URL to send new posts to. If not set then the bot will not be able to automatically send posts. |
| `$SHITPOST_SUFFIX` | `false` | An optional suffix to append to each post, usually to indicate that the post was generated. |
| `$SHITPOST_VISIBILITY` | `false` | A visibility to apply to all posts. Posts are public by default. Accepted value: `public`, `unlisted` or `private`. |
| `$SHITPOST_CW` | `false` | A content warning to apply to all posts. |

> [!NOTE]
> Compilation may take a long time, This is because requests for posts
> have to be sent one at a time (With each request retrieving 40
> posts).
>
> If you can solve [this](https://akko.wtf/notice/AiENPbvKa2ieHWjrJQ)
> problem, let me know, and I'll be able to dramatically speed up
> compile times.

<a id="format"></a>

## URL Formats

The URLs for `$SHITPOST_IN` and `$SHITPOST_OUT` should be API
endpoints for the Fediverse software you are using, generally these
will be of the form:

In: `https://<instance>/api/v1/accounts/<username or id>/statuses`

Out: `https://<instance>/api/v1/statuses`

> [!NOTE]
> The account the posts are sent to is not part of the URL and
> determined by the [private write key given at runtime](#command).

<a id="key"></a>

## Key Generation

shitpost-rs needs an API key in order to automatically send posts to a
Fedi accounts, as well as read non-private posts if you want to train
off of those.

shitpost-rs doesn't provide any authentication services. Instead you
need to provite your own. If you don't already have one, you can
generate one from one of these sites:

- Pleroma/Akkoma: https://prplecake.github.io/pleroma-access-token/
- Mastodon: https://takahashim.github.io/mastodon-access-token/
- *Key: Haven't found one yet, If you know of one, let me know.

> [!CAUTION]
> I'm not associated with either of these sites; It's your own
> responsibility to make sure you trust them, or for that matter, that
> you trust this. They're both open source, make your own judgements
> if you feel it necessary.

<a id="command"></a>

## Command Usage

Calling the `shitpost` binary without any arguments will simply print
the generated post to the terminal in HTML format.

In order to automatically send posts, a private key must be provided
as an argument, as it would be a bad idea to store it in the binary.

You can pass it as the first argument, or pass `-` as the first
argument, in which case it will read a key from `stdin`.
