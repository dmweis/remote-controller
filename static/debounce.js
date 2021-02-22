/**
 * Debounces websocket sends so that they only occur at max rate
 */
class Debouncer {
    constructor(connection) {
        this.connection = connection;
        this.interval_id = null;
        this.last_written = null;
    }

    /**
    * Internal method used for the event scheduler.
    */
    interval_event() {
        if (this.last_written) {
            this.connection.send(JSON.stringify(this.last_written));
            this.last_written = null;
        } else {
            clearInterval(this.interval_id);
            this.interval_id = null;
        }
    }

    /**
     * Sends or schedules a message for sending.
     */
    send(message) {
        if (this.interval_id == null) {
            this.connection.send(JSON.stringify(message));
            const captured = this;
            this.interval_id = setInterval(function () {
                captured.interval_event();
            }, 100);
        } else {
            this.last_written = message;
        }
    }
}