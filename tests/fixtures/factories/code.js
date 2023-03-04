import createFactory from './factory';
import {createEvent} from "effector";
import { createFactory2 } from './factory';

const x = createFactory(createEvent());

const x1 = createFactory(123);
const x2 = createFactory(456, 789);
const nested1 = createFactory(11, createFactory(111));
const nested2 = createFactory(22, createFactory2(222));
const nested3 = createFactory(22, createFactory2(222, createFactory(333)));

createFactory("no-assign");