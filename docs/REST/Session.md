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

### GET /v1/sessions
List all stored session IDs.

### Success
Response 200 JSON object

#### Example JSON object:
```json
{
  "total": 3,
  "ids": ["sess-123", "sess-456", "sess-789"]
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
