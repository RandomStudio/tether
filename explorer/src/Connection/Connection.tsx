import * as React from "react";

interface ConnectionProps {
  path: string;
  address: string;
  port: number;
}

interface ConnectionState {
  connected: boolean;
  sent: number;
  received: number;
}

class Connection extends React.Component<ConnectionProps, ConnectionState> {
  constructor(props: ConnectionProps) {
    super(props);
    this.state = {
      connected: false,
      sent: 0,
      received: 0,
    };
  }

  render() {
    return (
      <div>
        <h2>RabbitMQ Connection</h2>
        <div>
          <h3>Details</h3>
          <code>{JSON.stringify(this.props, null, 2)}</code>
        </div>

        <div>
          <h3>State</h3>
          <code>{JSON.stringify(this.state, null, 2)}</code>
        </div>
      </div>
    );
  }
}

export default Connection;
