import {do_action} from '../pkg/peregrine_standalone.js';

export class Communicate {
  isRecepientReady = false;

  outgoingQueue = [];

  waitingQueue = {};

  constructor() {
    this.ping();
  }

  private ping() {
    do_action({action: 'Initial', payload: {a: 1}, id: 1});
    this.onRecipientReady();
  }

  private onRecipientReady = () => {
    this.isRecepientReady = true;

  };

  private addToOutgoingQueue(task) {

    var task_id = this.outgoingQueue.length  + 1;

    this.outgoingQueue.push({action: task.action, payload: task.payload, id: task_id} );

    this.waitingQueue[task_id] = {
        callback: task.callback,
        id: task_id
    };

    this.processOutgoing();
  }

  private processOutgoing() {

    var task = this.outgoingQueue.shift();
    

    console.log(task)
    // TODO: Call the api handler on Rust with the task
    do_action(task);

    if(this.outgoingQueue.length){
        this.processOutgoing();
    }

  }


//   sendPostMessage(message) {
//     this.window.postMessage(message, "*");
//   }

//   subscribeToMessages() {
//     this.window.addEventListener("message", this.handleMessage);
//   }

private handleIncoming = (message) => {
    const {task_id, payload, error} = message;

    // TODO: Handle incoming messages without task IDs
    if (!task_id) {
      return;
    }

    const waitingAction = this.waitingQueue[task_id];

    delete this.waitingQueue[task_id];

    waitingAction.callback({
        payload,
        error
    });

  };


public send = (task) => {

    if (!this.isRecepientReady) {
      this.outgoingQueue.push(task);
    } else {
      this.addToOutgoingQueue(task);
    }
  };

}

export default Communicate;