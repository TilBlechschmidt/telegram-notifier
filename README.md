# Telegram Notifier

Very simple tool that takes HTTP POST requests at `/chat/:id` containing a JSON object with a `_message` field and sends it via Telegram.

The primary purpose for this is to send messages from InfluxDB 2.7 alerts to Telegram (by using the HTTP endpoint). The actual integration for Telegram is paywalled 🙄