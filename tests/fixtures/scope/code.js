import {createStore, createEvent} from 'effector'
import {useUnit} from "effector-react";
import {useUnit as useUnitSolid} from 'effector-solid';

const increment = createEvent()

const $counter = createStore(0)
    .on(increment, n => n + 1)

const counter = useUnit($counter);
const counter1 = useUnitSolid($counter);