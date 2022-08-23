import { createStore, createEvent } from 'effector';
import { useUnit } from "effector-react/scope";
import { useUnit as useUnitSolid } from "effector-solid/scope";
const increment = createEvent({
    sid: "h6036n5sjjl8",
    name: "increment"
});
const $counter = createStore(0, {
    sid: "2b9vnqi0nlc9c",
    name: "$counter"
}).on(increment, (n)=>n + 1);
const counter = useUnit($counter);
const counter1 = useUnitSolid($counter);
