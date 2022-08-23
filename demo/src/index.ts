import {createEvent, createStore} from 'effector';
import {factory} from "./factory";

const increment = createEvent();

console.log(createStore(createEvent()))
console.log(factory(increment));