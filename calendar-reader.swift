import EventKit
import Foundation

struct CalendarEvent: Codable {
    let title: String
    let start: String
    let end: String
    let allDay: Bool
    let calendar: String
}

let group = DispatchGroup()
let store = EKEventStore()

group.enter()
store.requestAccess(to: .event) { granted, error in
    if !granted {
        let errorOutput = "{\"error\": \"access denied\"}"
        FileHandle.standardError.write(errorOutput.data(using: .utf8)!)
        group.leave()
        return
    }

    let calendars = store.calendars(for: .event)
    let now = Date()
    let lookahead = CommandLine.arguments.count > 1
        ? Int(CommandLine.arguments[1]) ?? 14
        : 14
    let endDate = Calendar.current.date(byAdding: .day, value: lookahead, to: now)!
    let predicate = store.predicateForEvents(withStart: now, end: endDate, calendars: calendars)
    let events = store.events(matching: predicate)

    let dateFormatter = ISO8601DateFormatter()
    dateFormatter.formatOptions = [.withInternetDateTime]

    var results: [CalendarEvent] = []
    for event in events {
        let calEvent = CalendarEvent(
            title: event.title ?? "Unknown",
            start: dateFormatter.string(from: event.startDate),
            end: dateFormatter.string(from: event.endDate),
            allDay: event.isAllDay,
            calendar: event.calendar.title
        )
        results.append(calEvent)
    }

    let encoder = JSONEncoder()
    encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
    if let data = try? encoder.encode(results) {
        FileHandle.standardOutput.write(data)
    }

    group.leave()
}
group.wait()
