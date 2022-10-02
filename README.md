# botofgreed-rs

A Discord bot that allows you to rapidly retreive random messages from your meme channel(s)

<p align="center">
  <img alt="greed command" src="./assets/greed_command.png" width="45%"><br/>
  <img alt="sauce button" src="./assets/sauce_button.png" width="45%">
  <img alt="info command" src="./assets/info_command.png" width="45%">
</p>

### Usage

The Bot of Greed provides 3 global commands:
- `/greed`: Retreives a random message from the index and reposts the attached image
- `/memedex`: Refreshes the meme index/database (not a real DB)
- `/info`: Prints information about the Bot of Greed
  - version number
  - datetime of last index refresh
  - indexed message count (meme count)
  - uptime

Each meme sent by the Bot of Greed will have two Message Component Interaction buttons:
- `I'm feeling Greedy`: Identical to the `/greed` command, just triggers the bot again
- `sauce??`: Triggers a new message which contains the information about who originally posted the image the button was attached to

### Setup using docker
**coming soon**