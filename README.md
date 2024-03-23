# Oberon

<div align="center">
<img src="https://github.com/aaronsouthcombe/oberon/assets/141771153/2cd02000-7873-4d18-a5c8-e8422b83bf01" style="width:25%;">
</div>

### What is Oberon?


Oberon is a fast, asynchronous chat application that utilizes tokio and mpsc to divide and handle concurrent users into sessions and conversations and allows features such as registration, user access control, whispering and more.

### Current state of development:

#### Done:
- Client registration implemented
- Connections handled
- mpsc logic fully implemented
- Message handling
- Client connection
- Disconnection
#### To-do:
- Debugging and error handling
- Troubleshooting why some TCP clients receive messages and others don't
- PROTOBUFS!!!

### Fully functional using NetCat!
Connect to the server and the format is:
- First message is a string used to identify you
- All subsequent messages are formatted as follows: %%<destination>%%<message> where destination is the receiver's ID string previously specified.
