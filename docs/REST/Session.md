# REST Sessions API

## Table of contents
- [GET /v1/sessions](#get-/v1/sessions)
    - [Success](#success)
    - [Error](#errors)
- [GET /v1/sessions/{id}](#get-/v1/sessionsid)
    - [Success](#success-1)
    - [Error](#errors-1)

</details>

## Device Connection URL
http://{RAPID_ADDRESS}:{RAPID_PORT}<br>
(Default: http://{RAPID_ADDRESS}:27015)

## Resource: Session
The Session resource allows you to list and retrieve stored session data.
The session data includes information about the track, laps, and log points recorded during a driving session.
All the data is structured in JSON format.
The date values are represented in ISO 8601 format and described with the format "%Y-%m-%dT%H:%M:%S.%3f".
The id values are unique identifiers for each session and can be used to retrieve specific session details.

### GET /v1/sessions
List all stored session IDs.

### Success
Response 200 JSON object

#### Example JSON object:
```json
{
  "total": 3,
  "sessions":  [
    {
      "id": "sess-123",
      "date": "2012-04-23T18:25:43.511Z",
      "track": "Oschersleben",
      "laps": 12
    },
    {
      "id": "sess-456",
      "date": "2012-04-23T18:25:43.511Z",
      "track": "Oschersleben",
      "laps": 12
    },
    {
      "id": "sess-789",
      "date": "2012-04-23T18:25:43.511Z",
      "track": "Oschersleben",
      "laps": 12
    }
  ]
}
```

### Errors
- None

### GET /v1/sessions/{id}
Retrieve a single session by ID.

### Success
Response 200 JSON object

#### Example JSON oject
Example JSON object: 
```json
    {
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
                    "velocity": 100.0,
                    "longitude": 11.0,
                    "latitude": 52.0,
                    "time": "00:00:00.000",
                    "date": "01.01.1970"
                },
                {
                    "velocity": 100.0,
                    "longitude": 11.0,
                    "latitude": 52.0,
                    "time": "00:00:00.000",
                    "date": "01.01.1970"
                }
            ]
        }
    ]
    }
```

### Errors
- 404 for an invalid session ID.
