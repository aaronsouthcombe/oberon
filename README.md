# Oberon

<div align="center">
<img src="https://github.com/aaronsouthcombe/oberon/assets/141771153/2cd02000-7873-4d18-a5c8-e8422b83bf01" style="width:25%;">
</div>

### What is Oberon?


Oberon is a fast, asynchronous chat application that utilizes tokio and mpsc to divide and handle concurrent users into sessions and conversations and allows features such as registration, user access control, whispering and more.

It is not intended to actually be used in production environments, it was a fun project to explore more or less what apache kafka does. There is almost no error handling and no real return types due to this.

#### Features:
- Client registration 
- Connections 
- mpsc logic for message brokering
- Client connection

### Fully functional using NetCat!
Connect to the server and the format is:
- First message is a string used to identify you
- All subsequent messages are formatted as follows: %%destination%%message where destination is the receiver's ID string previously specified.

Note: The reason it's selective based on tcp clients is the format that they send messages with, e.g. \r\n\r\n vs \r\n or just plain \r.
