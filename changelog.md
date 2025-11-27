# Releases

## Release Note v0.4.1

### Continuous Integration
- add secret to to checkout

### Refactoring
- remove async from publish_event function



## Release Note v0.4.0

### Documentation
- document track detection events

### Features
- add publish event and wait for event function in module ctx
- add helper to get id and addr from a event
- add response constructor
- add request constructors
- make the module context cloneable

### Refactoring
- rename EventKindDiscriminant to EventKindType



## Release Note v0.3.1

### Continuous Integration
- stop dev branch experiment

### Refactoring
- refactor response handler in tests



## Release Note v0.3.0



## Release Note v0.3.0-dev19613284295

### Housekeeping
- set next development version
- avoid duplicating changelog header



## Release Note v0.1.0-dev19611809463

### Documentation
- document GET methods for the sessions resource

### Features
- SIGINT handler to correctly shutdown all modules
- provide session as REST resource
- REST interface for getting session ids
- introduce REST module

### Refactoring
- enable logging for tests
- debug log for in response handler



## Release Note v0.1.0-dev19603282309

### Continuous Integration
- use correct branch for bumping dev release
- enable linter action for the dev branch
- new action for bumping dev releases
- use correct body and tag name in bump release action
- run build action for develop branch also
- use correct variable name for the tag in auto bumping action



## Release Note v0.1.0

### Bug Fixes
- calculation of the next point
- Remove unused import from test_sessionfs_storage

### Continuous Integration
- for auto bumping use admin token
- use correct syntax for permissions
- give commitizen step content write acces
- github action for automatic version bump
- Introduce workflow for pr linting

### Features
- enable conventional commit check with commitizen
- use xdg_data_dirs in filesystem storage
- command line args for GNSS source
- Oschersleben GPS positions for the ConstantGnssSource
- introduce rapid_headless binary
- Handle only matching track detection responses
- handle only matching track detection responses
- store lap in active session when finished
- add new active session module
- load track async in laptimer
- new module for automatic track detection
- introduce ResponseHandler
- Introduce ResponseHandler
- generate discriminants and use them unit tests
- introduce load event for track ids
- port to new module based architecture
- add generic structs for request/response events
- port simple laptimer to module architecture
- Add events for laptimer module
- Move to new module architecture
- Introduce module_core The module_core defines the trait for modules and implements the event bus pattern for the communication between these modules

### Housekeeping
- add commitizen changelog template
- use custom commitizen configuration
- make package version compatible with commitizen cargo support
- add pre-commit install documentation
- add commitizen configuration file
- use workspace version number
- debug log for incoming requests and outgoing responses
- log used file system directories
- add debug log on detected track
- use tokio as workspace dependency
- use asset folder for tracks for testing
- Move modules in own directory and tests in own directory
- fix warnings in doc
- Fix dead code warnings
- remove unused imports
- remove not needed use directives
- push latest cargo lock file

### Refactoring
- make session in event shareable
- defaul trait for lap
- Use std::time::duration in lap
- prepare support for track storage
- better error reporting for events in tests


