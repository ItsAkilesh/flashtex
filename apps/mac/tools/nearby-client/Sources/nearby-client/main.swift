import Foundation
import NearbyClient

// `nearby-client pair|send|status|browse|forget …` — see NearbyCLI.usage.
setvbuf(stdout, nil, _IOLBF, 0)
let code = await NearbyCLI.run(Array(CommandLine.arguments.dropFirst())) { print($0) }
exit(code)
