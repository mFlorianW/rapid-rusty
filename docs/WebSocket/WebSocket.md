# WebSocket API
The WebSocket API provides real-time data streaming capabilities for connected devices.
These are primarily informations that either change often or it doesn't make sense to provide those via REST API.

## Table of contents
- [Device Connection URL](#device-connection-url)
- [Broadcast Events](#broadcast-events)
- [Live Session /v1/live_session](#live-session-v1live_session)
    - [Success](#success)
    - [Events](#events)
        - [Lap Started](#lap-started-broadcast)
        - [Sector finished](#sector-finished-broadcast)
        - [Lap finished](#lap-finished-broadcast)
        - [Current Laptime](#current-laptime-broadcast)
        - [Current Session](#current-session)
- [GNSS Data /v1/gnss_data](#gnss-data-v1gnss_data)
    - [Success](#success-1)
    - [Events](#events-1)
        - [GNSS Position](#gnss-position)
        - [GNSS Information](#gnss-information)
- [General /v1/general](#general-v1general)
    - [Success](#success-2)
    - [Events](#events-2)
        - [Track detected event](#track-detected-event)


## Device Connection URL
ws://{RAPID_ADDRESS}:{RAPID_PORT}</br>
Default: ws://10.0.0.1:27015

## Broadcast Events
All events described in this documentation that are broadcast events will have the following JSON format:

```json
{
  "event": "<event_name>",
  "data": {
    // event-specific data
  }
}
```
The "event" field indicates the type of event being sent.
The "data" field contains event-specific information, which may vary depending on the event type.

## Live Session /v1/live_session
The Live Session WebSocket endpoint provides real-time data streaming for an active driving session.

### Success
Response: 200

### Events
The following events are sent through the WebSocket connection during an active session:
All the events for this are broadcast events, that means there is no need for a subscription or request after the connection is established.
The time stamp values in the events will have the format "%H:%M:%S.%3f".

#### Lap Started (Broadcast)
The lap started event is sent when a new lap begins.
It contains no additional data.

Example JSON object:
```json
{
  "event": "lap_started"
  "data": {}
}
```
#### Sector finished (Broadcast)
The sector finished event is sent when a sector within a lap is completed.
It contains the sector number and the time taken to complete the sector.

Example JSON object:
```json
{
  "event": "sector_finished",
  "data": {
    "time": "00:45:123.456"
  }
}
```

#### Lap finished (Broadcast)
The lap finished event is sent when a lap is completed.
It contains the total lap time.

Example JSON object:
```json
{
  "event": "lap_finished",
  "data": {
    "time": "01:30:789.123"
  }
}
```

#### Current Laptime (Broadcast)
The current laptime event is sent periodically during a lap to provide the current lap time.
It contains the current absolute lap time since lap started event.

Example JSON object:
```json
{
  "event": "current_laptime",
  "data": {
    "time": "00:50:456.789"
  }
}

```
### Current Session
The current session event provides the complete data of the ongoing session.
It contains information about the track, laps, and log points recorded so far in the session.
This event is sent when a websocket connection is established to provide the current state of the session.
A client can use this data to synchronize its state with the ongoing session.
The client should not display any other data until this event is received to ensure consistency.

The data object in the event contains the full session data structured in the same as would be downloaded via the [REST API](https://github.com/mFlorianW/rapid-rusty/blob/main/docs/REST/Session.md#get-v1sessionsid).

Example JSON object:
```json
{
  "event": "current_session",
  "data": {
    "session": {
      "id": 0,
      "date": "01.01.1970",
      "time": "13:00:00.000",
      "track": {
        "name": "Oschersleben",
        "startline": {
          "latitude": 52.025833,
          "longitude": 11.279166
        },
        "finishline": {
          "latitude": 52.025833,
          "longitude": 11.279166
        },
        "sectors": [
          {
            "latitude": 52.025833,
            "longitude": 11.279166
          },
          {
            "latitude": 52.025833,
            "longitude": 11.279166
          }
        ]
      },
      "laps": [
        {
          "sectors": [
            "00:00:25.144",
            "00:00:25.144",
            "00:00:25.144",
            "00:00:25.144"
          ],
          "log_points": [
            {
              "velocity": 100,
              "longitude": 11,
              "latitude": 52,
              "time": "00:00:00.000",
              "date": "01.01.1970"
            },
            {
              "velocity": 100,
              "longitude": 11,
              "latitude": 52,
              "time": "00:00:00.000",
              "date": "01.01.1970"
            }
          ]
        }
      ]
    }
  }
}
```

## GNSS Data /v1/gnss_data
The GNSS Data WebSocket endpoint provides real-time GNSS (Global Navigation Satellite System) data from the connected device.

### Success
Response: 200

### Events
The following GNSS-related events are sent through the WebSocket connection.
All the events for this are broadcast events, that means there is no need for a subscription or request after the connection is established.

#### GNSS Position (Broadcast)
The GNSS position event provides real-time positional data from the GNSS receiver.
The provided latitude and longitude are in decimal degrees, speed is in meters per second, time is in the format "%H:%M:%S.%3f" and date in the format "dd.mm.YYYY".
Example JSON object:
```json
{
  "event": "gnss_position",
  "data": {
    "latitude": 37.7749,
    "longitude": -122.4194,
    "speed": 5.5,
    "time": "00:15:30.123",
    "datetime": "01.06.2024"
  }
}
```

#### GNSS Information (Broadcast)
The GNSS information event provides additional data about the GNSS receiver status.
Example JSON object:
```json
{
  "event": "gnss_information",
  "data": {
    "satellites": 8,
    "lock_status": "3D_FIX"
  }
}
```

## General /v1/general
The General WebSocket endpoint provides general real-time data from the connected device.

### Success
Response: 200

### Events
The following general events are sent through the WebSocket connection.
All the events for this are broadcast events, that means there is no need for a subscription or request after the connection is established.

#### Track detected event (Broadcast)
The track detected event is sent when the device detects a track.
```json
{
  "event": "track_detected",
  "data": {
    "track_name": "Silverstone Circuit",
  }
}
```
