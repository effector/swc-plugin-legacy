import {createDomain} from "effector";

const domain = createDomain();
const someDomain = createDomain();

const domainInner = someDomain.domain('inner domain');
const $store = domainInner.store(0);