function makeMap(str) {
	const map = Object.create(null);
	for (const key of str.split(',')) map[key] = 1;
	return (val) => (val in map);
}
const EMPTY_OBJ = {};
const EMPTY_ARR = [];
const NOOP = () => {};
const NO = () => false;
const isOn = (key) => key.charCodeAt(0) === 111 && key.charCodeAt(1) === 110 && (key.charCodeAt(2) > 122 || key.charCodeAt(2) < 97);
const isModelListener = (key) => key.startsWith('onUpdate:');
const extend = Object.assign;
const remove = (arr, el) => {
	const i = arr.indexOf(el);
	if (i > -1) {
		arr.splice(i, 1);
	}
};
const hasOwnProperty$1 = Object.prototype.hasOwnProperty;
const hasOwn = (val, key) => hasOwnProperty$1.call(val, key);
const isArray = Array.isArray;
const isMap = (val) => toTypeString(val) === '[object Map]';
const isSet = (val) => toTypeString(val) === '[object Set]';
const isFunction = (val) => typeof val === 'function';
const isString = (val) => typeof val === 'string';
const isSymbol = (val) => typeof val === 'symbol';
const isObject = (val) => val !== null && typeof val === 'object';
const isPromise = (val) => {
	return (isObject(val) || isFunction(val)) && isFunction(val.then) && isFunction(val.catch);
};
const objectToString = Object.prototype.toString;
const toTypeString = (value) => objectToString.call(value);
const toRawType = (value) => {
	return toTypeString(value).slice(8, -1);
};
const isPlainObject = (val) => toTypeString(val) === '[object Object]';
const isIntegerKey = (key) => isString(key) && key !== 'NaN' && key[0] !== '-' && '' + parseInt(key, 10) === key;
const isReservedProp = makeMap(',key,ref,ref_for,ref_key,onVnodeBeforeMount,onVnodeMounted,onVnodeBeforeUpdate,onVnodeUpdated,onVnodeBeforeUnmount,onVnodeUnmounted');
const cacheStringFunction = (fn) => {
	const cache = Object.create(null);
	return (str) => {
		const hit = cache[str];
		return hit || (cache[str] = fn(str));
	};
};
const camelizeRE = /-(\w)/g;
const camelize = cacheStringFunction((str) => {
	return str.replace(camelizeRE, (__unused_B052, c) => c ? c.toUpperCase() : '');
});
const hyphenateRE = /\B([A-Z])/g;
const hyphenate = cacheStringFunction((str) => str.replace(hyphenateRE, '-$1').toLowerCase());
const capitalize = cacheStringFunction((str) => {
	return str.charAt(0).toUpperCase() + str.slice(1);
});
const toHandlerKey = cacheStringFunction((str) => {
	const s = str ? `on${capitalize(str)}` : '';
	return s;
});
const hasChanged = (value, oldValue) => !Object.is(value, oldValue);
const invokeArrayFns = (fns, ...arg) => {
	for (let i = 0; i < fns.length; i++) {
		fns[i](...arg);
	}
};
const def = (obj, key, value, writable = false) => {
	Object.defineProperty(obj, key, {
		configurable: true,
		enumerable: false,
		writable,
		value
	});
};
const looseToNumber = (val) => {
	const n = parseFloat(val);
	return isNaN(n) ? val : n;
};
let _globalThis;
const getGlobalThis = () => {
	return _globalThis || (_globalThis = typeof globalThis !== 'undefined' ? globalThis : typeof self !== 'undefined' ? self : typeof window !== 'undefined' ? window : typeof global !== 'undefined' ? global : {});
};
function normalizeStyle(value) {
	if (isArray(value)) {
		const res = {};
		for (let i = 0; i < value.length; i++) {
			const item = value[i];
			const normalized = isString(item) ? parseStringStyle(item) : normalizeStyle(item);
			if (normalized) {
				for (const key in normalized) {
					res[key] = normalized[key];
				}
			}
		}
		return res;
	} else if (isString(value) || isObject(value)) {
		return value;
	}
}
const listDelimiterRE = /;(?![^(]*\))/g;
const propertyDelimiterRE = /:([^]+)/;
const styleCommentRE = /\/\*[^]*?\*\//g;
function parseStringStyle(cssText) {
	const ret = {};
	cssText.replace(styleCommentRE, '').split(listDelimiterRE).forEach((item) => {
		if (item) {
			const tmp = item.split(propertyDelimiterRE);
			tmp.length > 1 && (ret[tmp[0].trim()] = tmp[1].trim());
		}
	});
	return ret;
}
function normalizeClass(value) {
	let res = '';
	if (isString(value)) {
		res = value;
	} else if (isArray(value)) {
		for (let i = 0; i < value.length; i++) {
			const normalized = normalizeClass(value[i]);
			if (normalized) {
				res += normalized + ' ';
			}
		}
	} else if (isObject(value)) {
		for (const name in value) {
			if (value[name]) {
				res += name + ' ';
			}
		}
	}
	return res.trim();
}
const specialBooleanAttrs = 'itemscope,allowfullscreen,formnovalidate,ismap,nomodule,novalidate,readonly';
const isSpecialBooleanAttr = makeMap(specialBooleanAttrs);
function includeBooleanAttr(value) {
	return !!value || value === '';
}
let activeEffectScope;
class EffectScope {
	constructor(detached = false) {
		this.detached = detached;
		this._active = true;
		this.effects = [];
		this.cleanups = [];
		this._isPaused = false;
		this.parent = activeEffectScope;
		if (!detached && activeEffectScope) {
			this.index = (activeEffectScope.scopes || (activeEffectScope.scopes = [])).push(this) - 1;
		}
	}
	get active() {
		return this._active;
	}
	pause() {
		if (this._active) {
			this._isPaused = true;
			let i, l;
			if (this.scopes) {
				for (i = 0, l = this.scopes.length; i < l; i++) {
					this.scopes[i].pause();
				}
			}
			for (i = 0, l = this.effects.length; i < l; i++) {
				this.effects[i].pause();
			}
		}
	}
	resume() {
		if (this._active) {
			if (this._isPaused) {
				this._isPaused = false;
				let i, l;
				if (this.scopes) {
					for (i = 0, l = this.scopes.length; i < l; i++) {
						this.scopes[i].resume();
					}
				}
				for (i = 0, l = this.effects.length; i < l; i++) {
					this.effects[i].resume();
				}
			}
		}
	}
	run(fn) {
		if (this._active) {
			const currentEffectScope = activeEffectScope;
			try {
				activeEffectScope = this;
				return fn();
			} finally {
				activeEffectScope = currentEffectScope;
			}
		}
	}
	on() {
		activeEffectScope = this;
	}
	off() {
		activeEffectScope = this.parent;
	}
	stop(fromParent) {
		if (this._active) {
			let i, l;
			for (i = 0, l = this.effects.length; i < l; i++) {
				this.effects[i].stop();
			}
			for (i = 0, l = this.cleanups.length; i < l; i++) {
				this.cleanups[i]();
			}
			if (this.scopes) {
				for (i = 0, l = this.scopes.length; i < l; i++) {
					this.scopes[i].stop(true);
				}
			}
			if (!this.detached && this.parent && !fromParent) {
				const last = this.parent.scopes.pop();
				if (last && last !== this) {
					this.parent.scopes[this.index] = last;
					last.index = this.index;
				}
			}
			this.parent = undefined;
			this._active = false;
		}
	}
}
function getCurrentScope() {
	return activeEffectScope;
}
let activeSub;
const pausedQueueEffects = new WeakSet();
class ReactiveEffect {
	constructor(fn) {
		this.fn = fn;
		this.deps = undefined;
		this.depsTail = undefined;
		this.flags = 1 | 4;
		this.next = undefined;
		this.cleanup = undefined;
		this.scheduler = undefined;
		if (activeEffectScope && activeEffectScope.active) {
			activeEffectScope.effects.push(this);
		}
	}
	pause() {
		this.flags |= 64;
	}
	resume() {
		if (this.flags & 64) {
			this.flags &= ~64;
			if (pausedQueueEffects.has(this)) {
				pausedQueueEffects.delete(this);
				this.trigger();
			}
		}
	}
	notify() {
		if (this.flags & 2 && !(this.flags & 32)) {
			return;
		}
		if (!(this.flags & 8)) {
			batch(this);
		}
	}
	run() {
		if (!(this.flags & 1)) {
			return this.fn();
		}
		this.flags |= 2;
		cleanupEffect(this);
		prepareDeps(this);
		const prevEffect = activeSub;
		const prevShouldTrack = shouldTrack;
		activeSub = this;
		shouldTrack = true;
		try {
			return this.fn();
		} finally {
			cleanupDeps(this);
			activeSub = prevEffect;
			shouldTrack = prevShouldTrack;
			this.flags &= ~2;
		}
	}
	stop() {
		if (this.flags & 1) {
			for (let link = this.deps; link; link = link.nextDep) {
				removeSub(link);
			}
			this.deps = (this.depsTail = undefined, undefined);
			cleanupEffect(this);
			this.onStop && this.onStop();
			this.flags &= ~1;
		}
	}
	trigger() {
		if (this.flags & 64) {
			pausedQueueEffects.add(this);
		} else if (this.scheduler) {
			this.scheduler();
		} else {
			this.runIfDirty();
		}
	}
	runIfDirty() {
		if (isDirty(this)) {
			this.run();
		}
	}
	get dirty() {
		return isDirty(this);
	}
}
let batchDepth = 0;
let batchedSub;
let batchedComputed;
function batch(sub, isComputed = false) {
	sub.flags |= 8;
	if (isComputed) {
		sub.next = batchedComputed;
		batchedComputed = sub;
		return;
	}
	sub.next = batchedSub;
	batchedSub = sub;
}
function startBatch() {
	batchDepth++;
}
function endBatch() {
	if (--batchDepth > 0) {
		return;
	}
	if (batchedComputed) {
		let e = batchedComputed;
		batchedComputed = undefined;
		while (e) {
			const next = e.next;
			e.next = undefined;
			e.flags &= ~8;
			e = next;
		}
	}
	let error;
	while (batchedSub) {
		let e = batchedSub;
		batchedSub = undefined;
		while (e) {
			const next = e.next;
			e.next = undefined;
			e.flags &= ~8;
			if (e.flags & 1) {
				try {
					e.trigger();
				} catch (err) {
					if (!error) error = err;
				}
			}
			e = next;
		}
	}
	if (error) throw error;
}
function prepareDeps(sub) {
	for (let link = sub.deps; link; link = link.nextDep) {
		link.version = -1;
		link.prevActiveLink = link.dep.activeLink;
		link.dep.activeLink = link;
	}
}
function cleanupDeps(sub) {
	let head;
	let tail = sub.depsTail;
	let link = tail;
	while (link) {
		const prev = link.prevDep;
		if (link.version === -1) {
			if (link === tail) tail = prev;
			removeSub(link);
			removeDep(link);
		} else {
			head = link;
		}
		link.dep.activeLink = link.prevActiveLink;
		link.prevActiveLink = undefined;
		link = prev;
	}
	sub.deps = head;
	sub.depsTail = tail;
}
function isDirty(sub) {
	for (let link = sub.deps; link; link = link.nextDep) {
		if (link.dep.version, link.version, link.dep.computed && (refreshComputed(link.dep.computed) || (link.dep.version, link.version, false))) {
			return true;
		}
	}
	if (sub._dirty) {
		return true;
	}
	return false;
}
function refreshComputed(computed2) {
	if (computed2.flags & 4 && !(computed2.flags & 16)) {
		return;
	}
	computed2.flags &= ~16;
	if (computed2.globalVersion === globalVersion) {
		return;
	}
	computed2.globalVersion = 0;
	const dep = computed2.dep;
	computed2.flags |= 2;
	if (dep.version > 0 && !computed2.isSSR && computed2.deps && !isDirty(computed2)) {
		computed2.flags &= ~2;
		return;
	}
	const prevSub = activeSub;
	const prevShouldTrack = shouldTrack;
	activeSub = computed2;
	shouldTrack = true;
	try {
		prepareDeps(computed2);
		const value = computed2.fn(computed2._value);
		if (dep.version === 0 || hasChanged(value, computed2._value)) {
			computed2._value = value;
			dep.version++;
		}
	} catch (err) {
		dep.version++;
		throw err;
	} finally {
		activeSub = prevSub;
		shouldTrack = prevShouldTrack;
		cleanupDeps(computed2);
		computed2.flags &= ~2;
	}
}
function removeSub(link, soft = false) {
	const { dep, prevSub, nextSub } = link;
	if (prevSub) {
		prevSub.nextSub = nextSub;
		link.prevSub = undefined;
	}
	if (nextSub) {
		nextSub.prevSub = prevSub;
		link.nextSub = undefined;
	}
	if (dep.subs === link) {
		dep.subs = prevSub;
		if (!prevSub && dep.computed) {
			dep.computed.flags &= ~4;
			for (let l = dep.computed.deps; l; l = l.nextDep) {
				removeSub(l, true);
			}
		}
	}
	if (!soft && !--dep.sc && dep.map) {
		dep.map.delete(dep.key);
	}
}
function removeDep(link) {
	const { prevDep, nextDep } = link;
	if (prevDep) {
		prevDep.nextDep = nextDep;
		link.prevDep = undefined;
	}
	if (nextDep) {
		nextDep.prevDep = prevDep;
		link.nextDep = undefined;
	}
}
let shouldTrack = true;
const trackStack = [];
function pauseTracking() {
	trackStack.push(shouldTrack);
	shouldTrack = false;
}
function resetTracking() {
	const last = trackStack.pop();
	shouldTrack = last === undefined ? true : last;
}
function cleanupEffect(e) {
	const { cleanup } = e;
	e.cleanup = undefined;
	if (cleanup) {
		const prevSub = activeSub;
		activeSub = undefined;
		try {
			cleanup();
		} finally {
			activeSub = prevSub;
		}
	}
}
let globalVersion = 0;
class Link {
	constructor(sub, dep) {
		this.sub = sub;
		this.dep = dep;
		this.version = dep.version;
		this.nextDep = (this.prevDep = (this.nextSub = (this.prevSub = (this.prevActiveLink = undefined, undefined), undefined), undefined), undefined);
	}
}
class Dep {
	constructor(computed2) {
		this.computed = computed2;
		this.version = 0;
		this.activeLink = undefined;
		this.subs = undefined;
		this.map = undefined;
		this.key = undefined;
		this.sc = 0;
	}
	track() {
		if (!activeSub || !shouldTrack || (this.computed, true)) {
			return;
		}
		let link = this.activeLink;
		if (link === undefined || (link.sub, false)) {
			link = this.activeLink = new Link(activeSub, this);
			if (!activeSub.deps) {
				activeSub.deps = activeSub.depsTail = link;
			} else {
				link.prevDep = activeSub.depsTail;
				activeSub.depsTail.nextDep = link;
				activeSub.depsTail = link;
			}
			addSub(link);
		} else if (link.version === -1) {
			link.version = this.version;
			if (link.nextDep) {
				const next = link.nextDep;
				next.prevDep = link.prevDep;
				if (link.prevDep) {
					link.prevDep.nextDep = next;
				}
				link.prevDep = activeSub.depsTail;
				link.nextDep = undefined;
				activeSub.depsTail.nextDep = link;
				activeSub.depsTail = link;
				if (activeSub.deps === link) {
					activeSub.deps = next;
				}
			}
		}
		return link;
	}
	trigger(debugInfo) {
		this.version++;
		globalVersion++;
		this.notify(debugInfo);
	}
	notify() {
		startBatch();
		try {
			for (let link = this.subs; link; link = link.prevSub) {
				if (link.sub.notify()) {
					link.sub.dep.notify();
				}
			}
		} finally {
			endBatch();
		}
	}
}
function addSub(link) {
	link.dep.sc++;
	if (link.sub.flags & 4) {
		const computed2 = link.dep.computed;
		if (computed2 && !link.dep.subs) {
			computed2.flags |= 4 | 16;
			for (let l = computed2.deps; l; l = l.nextDep) {
				addSub(l);
			}
		}
		const currentTail = link.dep.subs;
		if (currentTail !== link) {
			link.prevSub = currentTail;
			if (currentTail) currentTail.nextSub = link;
		}
		link.dep.subs = link;
	}
}
const targetMap = new WeakMap();
const ITERATE_KEY = Symbol('');
const MAP_KEY_ITERATE_KEY = Symbol('');
const ARRAY_ITERATE_KEY = Symbol('');
function track(target, __unused_45DA, key) {
	if (shouldTrack && activeSub) {
		let depsMap = targetMap.get(target);
		if (!depsMap) {
			targetMap.set(target, depsMap = new Map());
		}
		let dep = depsMap.get(key);
		if (!dep) {
			depsMap.set(key, dep = new Dep());
			dep.map = depsMap;
			dep.key = key;
		}
		{
			dep.track();
		}
	}
}
function trigger(target, type, key, newValue) {
	const depsMap = targetMap.get(target);
	if (!depsMap) {
		globalVersion++;
		return;
	}
	const run = (dep) => {
		if (dep) {
			{
				dep.trigger();
			}
		}
	};
	startBatch();
	if (type === 'clear') {
		depsMap.forEach(run);
	} else {
		const targetIsArray = isArray(target);
		const isArrayIndex = targetIsArray && isIntegerKey(key);
		if (targetIsArray && key === 'length') {
			const newLength = Number(newValue);
			depsMap.forEach((dep, key2) => {
				if (key2 === 'length' || key2 === ARRAY_ITERATE_KEY || !isSymbol(key2) && key2 >= newLength) {
					run(dep);
				}
			});
		} else {
			if (key !== undefined || depsMap.has(undefined)) {
				run(depsMap.get(key));
			}
			if (isArrayIndex) {
				run(depsMap.get(ARRAY_ITERATE_KEY));
			}
			switch (type) {
				case 'add':
					if (!targetIsArray) {
						run(depsMap.get(ITERATE_KEY));
						if (isMap(target)) {
							run(depsMap.get(MAP_KEY_ITERATE_KEY));
						}
					} else if (isArrayIndex) {
						run(depsMap.get('length'));
					}
					break;
				case 'delete':
					if (!targetIsArray) {
						run(depsMap.get(ITERATE_KEY));
						if (isMap(target)) {
							run(depsMap.get(MAP_KEY_ITERATE_KEY));
						}
					}
					break;
				case 'set':
					if (isMap(target)) {
						run(depsMap.get(ITERATE_KEY));
					}
					break;
			}
		}
	}
	endBatch();
}
function reactiveReadArray(array) {
	const raw = toRaw(array);
	if (raw === array) return raw;
	track(raw, 0, ARRAY_ITERATE_KEY);
	return isShallow(array) ? raw : raw.map(toReactive);
}
function shallowReadArray(arr) {
	track(arr = toRaw(arr), 0, ARRAY_ITERATE_KEY);
	return arr;
}
const arrayInstrumentations = {
	__proto__: null,
	[Symbol.iterator]() {
		return iterator(this, Symbol.iterator, toReactive);
	},
	concat(...args) {
		return reactiveReadArray(this).concat(...args.map((x) => isArray(x) ? reactiveReadArray(x) : x));
	},
	entries() {
		return iterator(this, 'entries', (value) => {
			value[1] = toReactive(value[1]);
			return value;
		});
	},
	every(fn, thisArg) {
		return apply(this, 'every', fn, thisArg, undefined, arguments);
	},
	filter(fn, thisArg) {
		return apply(this, 'filter', fn, thisArg, (v) => v.map(toReactive), arguments);
	},
	find(fn, thisArg) {
		return apply(this, 'find', fn, thisArg, toReactive, arguments);
	},
	findIndex(fn, thisArg) {
		return apply(this, 'findIndex', fn, thisArg, undefined, arguments);
	},
	findLast(fn, thisArg) {
		return apply(this, 'findLast', fn, thisArg, toReactive, arguments);
	},
	findLastIndex(fn, thisArg) {
		return apply(this, 'findLastIndex', fn, thisArg, undefined, arguments);
	},
	forEach(fn, thisArg) {
		return apply(this, 'forEach', fn, thisArg, undefined, arguments);
	},
	includes(...args) {
		return searchProxy(this, 'includes', args);
	},
	indexOf(...args) {
		return searchProxy(this, 'indexOf', args);
	},
	join(separator) {
		return reactiveReadArray(this).join(separator);
	},
	lastIndexOf(...args) {
		return searchProxy(this, 'lastIndexOf', args);
	},
	map(fn, thisArg) {
		return apply(this, 'map', fn, thisArg, undefined, arguments);
	},
	pop() {
		return noTracking(this, 'pop');
	},
	push(...args) {
		return noTracking(this, 'push', args);
	},
	reduce(fn, ...args) {
		return reduce(this, 'reduce', fn, args);
	},
	reduceRight(fn, ...args) {
		return reduce(this, 'reduceRight', fn, args);
	},
	shift() {
		return noTracking(this, 'shift');
	},
	some(fn, thisArg) {
		return apply(this, 'some', fn, thisArg, undefined, arguments);
	},
	splice(...args) {
		return noTracking(this, 'splice', args);
	},
	toReversed() {
		return reactiveReadArray(this).toReversed();
	},
	toSorted(comparer) {
		return reactiveReadArray(this).toSorted(comparer);
	},
	toSpliced(...args) {
		return reactiveReadArray(this).toSpliced(...args);
	},
	unshift(...args) {
		return noTracking(this, 'unshift', args);
	},
	values() {
		return iterator(this, 'values', toReactive);
	}
};
function iterator(self2, method, wrapValue) {
	const arr = shallowReadArray(self2);
	const iter = arr[method]();
	if (arr !== self2 && !isShallow(self2)) {
		iter._next = iter.next;
		iter.next = () => {
			const result = iter._next();
			if (result.value) {
				result.value = wrapValue(result.value);
			}
			return result;
		};
	}
	return iter;
}
const arrayProto = Array.prototype;
function apply(self2, method, fn, thisArg, wrappedRetFn, args) {
	const arr = shallowReadArray(self2);
	const needsWrap = arr !== self2 && !isShallow(self2);
	const methodFn = arr[method];
	if (methodFn !== arrayProto[method]) {
		const result2 = methodFn.apply(self2, args);
		return needsWrap ? toReactive(result2) : result2;
	}
	let wrappedFn = fn;
	if (arr !== self2) {
		if (needsWrap) {
			wrappedFn = function(item, index) {
				return fn.call(this, toReactive(item), index, self2);
			};
		} else if (fn.length > 2) {
			wrappedFn = function(item, index) {
				return fn.call(this, item, index, self2);
			};
		}
	}
	const result = methodFn.call(arr, wrappedFn, thisArg);
	return needsWrap && wrappedRetFn ? wrappedRetFn(result) : result;
}
function reduce(self2, method, fn, args) {
	const arr = shallowReadArray(self2);
	let wrappedFn = fn;
	if (arr !== self2) {
		if (!isShallow(self2)) {
			wrappedFn = function(acc, item, index) {
				return fn.call(this, acc, toReactive(item), index, self2);
			};
		} else if (fn.length > 3) {
			wrappedFn = function(acc, item, index) {
				return fn.call(this, acc, item, index, self2);
			};
		}
	}
	return arr[method](wrappedFn, ...args);
}
function searchProxy(self2, method, args) {
	const arr = toRaw(self2);
	track(arr, 0, ARRAY_ITERATE_KEY);
	const res = arr[method](...args);
	if ((res === -1 || res === false) && isProxy(args[0])) {
		args[0] = toRaw(args[0]);
		return arr[method](...args);
	}
	return res;
}
function noTracking(self2, method, args = []) {
	pauseTracking();
	startBatch();
	const res = toRaw(self2)[method].apply(self2, args);
	endBatch();
	resetTracking();
	return res;
}
const isNonTrackableKeys = makeMap('__proto__,__v_isRef,__isVue');
const builtInSymbols = new Set(Object.getOwnPropertyNames(Symbol).filter((key) => key !== 'arguments' && key !== 'caller').map((key) => Symbol[key]).filter(isSymbol));
function hasOwnProperty(key) {
	if (!isSymbol(key)) key = String(key);
	const obj = toRaw(this);
	track(obj, 0, key);
	return obj.hasOwnProperty(key);
}
class BaseReactiveHandler {
	constructor(_isReadonly = false, _isShallow = false) {
		this._isReadonly = _isReadonly;
		this._isShallow = _isShallow;
	}
	get(target, key, receiver) {
		const isReadonly2 = this._isReadonly, isShallow2 = this._isShallow;
		if (key === '__v_isReactive') {
			return !isReadonly2;
		} else if (key === '__v_isReadonly') {
			return isReadonly2;
		} else if (key === '__v_isShallow') {
			return isShallow2;
		} else if (key === '__v_raw') {
			if (receiver === (isReadonly2 ? isShallow2 ? shallowReadonlyMap : readonlyMap : isShallow2 ? shallowReactiveMap : reactiveMap).get(target) || (Object.getPrototypeOf(target), Object.getPrototypeOf(receiver), true)) {
				return target;
			}
			return;
		}
		const targetIsArray = isArray(target);
		if (!isReadonly2) {
			let fn;
			if (targetIsArray && (fn = arrayInstrumentations[key])) {
				return fn;
			}
			if (key === 'hasOwnProperty') {
				return hasOwnProperty;
			}
		}
		const res = Reflect.get(target, key, isRef(target) ? target : receiver);
		if (isSymbol(key) ? builtInSymbols.has(key) : isNonTrackableKeys(key)) {
			return res;
		}
		if (!isReadonly2) {
			track(target, 0, key);
		}
		if (isShallow2) {
			return res;
		}
		if (isRef(res)) {
			return targetIsArray && isIntegerKey(key) ? res : res.value;
		}
		if (isObject(res)) {
			return isReadonly2 ? readonly(res) : reactive(res);
		}
		return res;
	}
}
class MutableReactiveHandler extends BaseReactiveHandler {
	constructor(isShallow2 = false) {
		super(false, isShallow2);
	}
	set(target, key, value, receiver) {
		let oldValue = target[key];
		if (!this._isShallow) {
			const isOldValueReadonly = isReadonly(oldValue);
			if (!isShallow(value) && !isReadonly(value)) {
				oldValue = toRaw(oldValue);
				value = toRaw(value);
			}
			if (!isArray(target) && isRef(oldValue) && !isRef(value)) {
				if (isOldValueReadonly) {
					return false;
				} else {
					oldValue.value = value;
					return true;
				}
			}
		}
		const hadKey = isArray(target) && isIntegerKey(key) ? Number(key) < target.length : hasOwn(target, key);
		const result = Reflect.set(target, key, value, isRef(target) ? target : receiver);
		if (target === toRaw(receiver)) {
			if (!hadKey) {
				trigger(target, 'add', key, value);
			} else if (hasChanged(value, oldValue)) {
				trigger(target, 'set', key, value);
			}
		}
		return result;
	}
	deleteProperty(target, key) {
		const hadKey = hasOwn(target, key);
		target[key];
		const result = Reflect.deleteProperty(target, key);
		if (result && hadKey) {
			trigger(target, 'delete', key, undefined);
		}
		return result;
	}
	has(target, key) {
		const result = Reflect.has(target, key);
		if (!isSymbol(key) || !builtInSymbols.has(key)) {
			track(target, 0, key);
		}
		return result;
	}
	ownKeys(target) {
		track(target, 0, isArray(target) ? 'length' : ITERATE_KEY);
		return Reflect.ownKeys(target);
	}
}
class ReadonlyReactiveHandler extends BaseReactiveHandler {
	constructor(isShallow2 = false) {
		super(true, isShallow2);
	}
	set() {
		return true;
	}
	deleteProperty() {
		return true;
	}
}
const mutableHandlers = new MutableReactiveHandler();
const readonlyHandlers = new ReadonlyReactiveHandler();
const shallowReactiveHandlers = new MutableReactiveHandler(true);
const __unused_3CB6 = new ReadonlyReactiveHandler(true);
const toShallow = (value) => value;
const getProto = (v) => Reflect.getPrototypeOf(v);
function createIterableMethod(method, isReadonly2, isShallow2) {
	return function(...args) {
		const target = this.__v_raw;
		const rawTarget = toRaw(target);
		const targetIsMap = isMap(rawTarget);
		const isPair = method === 'entries' || method === Symbol.iterator && targetIsMap;
		const isKeyOnly = method === 'keys' && targetIsMap;
		const innerIterator = target[method](...args);
		const wrap = isShallow2 ? toShallow : isReadonly2 ? toReadonly : toReactive;
		!isReadonly2 && track(rawTarget, 0, isKeyOnly ? MAP_KEY_ITERATE_KEY : ITERATE_KEY);
		return {
			next() {
				const { value, done } = innerIterator.next();
				return done ? {
					value,
					done
				} : {
					value: isPair ? [wrap(value[0]), wrap(value[1])] : wrap(value),
					done
				};
			},
			[Symbol.iterator]() {
				return this;
			}
		};
	};
}
function createReadonlyMethod() {
	return function() {
		return type === 'delete' ? false : type === 'clear' ? undefined : this;
	};
}
function createInstrumentations(readonly2, shallow) {
	const instrumentations = {
		get(key) {
			const target = this.__v_raw;
			const rawTarget = toRaw(target);
			const rawKey = toRaw(key);
			if (!readonly2) {
				if (hasChanged(key, rawKey)) {
					track(rawTarget, 0, key);
				}
				track(rawTarget, 0, rawKey);
			}
			const { has } = getProto(rawTarget);
			const wrap = shallow ? toShallow : readonly2 ? toReadonly : toReactive;
			if (has.call(rawTarget, key)) {
				return wrap(target.get(key));
			} else if (has.call(rawTarget, rawKey)) {
				return wrap(target.get(rawKey));
			} else if (target !== rawTarget) {
				target.get(key);
			}
		},
		get size() {
			const target = this.__v_raw;
			!readonly2 && track(toRaw(target), 0, ITERATE_KEY);
			return Reflect.get(target, 'size', target);
		},
		has(key) {
			const target = this.__v_raw;
			const rawTarget = toRaw(target);
			const rawKey = toRaw(key);
			if (!readonly2) {
				if (hasChanged(key, rawKey)) {
					track(rawTarget, 0, key);
				}
				track(rawTarget, 0, rawKey);
			}
			return key === rawKey ? target.has(key) : target.has(key) || target.has(rawKey);
		},
		forEach(callback, thisArg) {
			const observed = this;
			const target = observed.__v_raw;
			const rawTarget = toRaw(target);
			const wrap = shallow ? toShallow : readonly2 ? toReadonly : toReactive;
			!readonly2 && track(rawTarget, 0, ITERATE_KEY);
			return target.forEach((value, key) => {
				return callback.call(thisArg, wrap(value), wrap(key), observed);
			});
		}
	};
	extend(instrumentations, readonly2 ? {
		add: createReadonlyMethod(),
		set: createReadonlyMethod(),
		delete: createReadonlyMethod(),
		clear: createReadonlyMethod()
	} : {
		add(value) {
			if (!shallow && !isShallow(value) && !isReadonly(value)) {
				value = toRaw(value);
			}
			const target = toRaw(this);
			const proto = getProto(target);
			const hadKey = proto.has.call(target, value);
			if (!hadKey) {
				target.add(value);
				trigger(target, 'add', value, value);
			}
			return this;
		},
		set(key, value) {
			if (!shallow && !isShallow(value) && !isReadonly(value)) {
				value = toRaw(value);
			}
			const target = toRaw(this);
			const { has, get } = getProto(target);
			let hadKey = has.call(target, key);
			if (!hadKey) {
				key = toRaw(key);
				hadKey = has.call(target, key);
			}
			const oldValue = get.call(target, key);
			target.set(key, value);
			if (!hadKey) {
				trigger(target, 'add', key, value);
			} else if (hasChanged(value, oldValue)) {
				trigger(target, 'set', key, value);
			}
			return this;
		},
		delete(key) {
			const target = toRaw(this);
			const { has, get } = getProto(target);
			let hadKey = has.call(target, key);
			if (!hadKey) {
				key = toRaw(key);
				hadKey = has.call(target, key);
			}
			get && get.call(target, key);
			const result = target.delete(key);
			if (hadKey) {
				trigger(target, 'delete', key, undefined);
			}
			return result;
		},
		clear() {
			const target = toRaw(this);
			const hadItems = target.size !== 0;
			const result = target.clear();
			if (hadItems) {
				trigger(target);
			}
			return result;
		}
	});
	const iteratorMethods = [
		'keys',
		'values',
		'entries',
		Symbol.iterator
	];
	iteratorMethods.forEach((method) => {
		instrumentations[method] = createIterableMethod(method, readonly2, shallow);
	});
	return instrumentations;
}
function createInstrumentationGetter(isReadonly2, shallow) {
	const instrumentations = createInstrumentations(isReadonly2, shallow);
	return (target, key, receiver) => {
		if (key === '__v_isReactive') {
			return !isReadonly2;
		} else if (key === '__v_isReadonly') {
			return isReadonly2;
		} else if (key === '__v_raw') {
			return target;
		}
		return Reflect.get(hasOwn(instrumentations, key) && key in target ? instrumentations : target, key, receiver);
	};
}
const mutableCollectionHandlers = { get: createInstrumentationGetter(false, false) };
const shallowCollectionHandlers = { get: createInstrumentationGetter(false, true) };
const readonlyCollectionHandlers = { get: createInstrumentationGetter(true, false) };
const __unused_2885 = { get: createInstrumentationGetter(true, true) };
const reactiveMap = new WeakMap();
const shallowReactiveMap = new WeakMap();
const readonlyMap = new WeakMap();
const shallowReadonlyMap = new WeakMap();
function targetTypeMap(rawType) {
	switch (rawType) {
		case 'Object':
		case 'Array': return 1;
		case 'Map':
		case 'Set':
		case 'WeakMap':
		case 'WeakSet': return 2;
		default: return 0;
	}
}
function getTargetType(value) {
	return value.__v_skip || !Object.isExtensible(value) ? 0 : targetTypeMap(toRawType(value));
}
function reactive(target) {
	if (isReadonly(target)) {
		return target;
	}
	return createReactiveObject(target, false, mutableHandlers, mutableCollectionHandlers, reactiveMap);
}
function shallowReactive(target) {
	return createReactiveObject(target, false, shallowReactiveHandlers, shallowCollectionHandlers, shallowReactiveMap);
}
function readonly(target) {
	return createReactiveObject(target, true, readonlyHandlers, readonlyCollectionHandlers, readonlyMap);
}
function createReactiveObject(target, isReadonly2, baseHandlers, collectionHandlers, proxyMap) {
	if (!isObject(target)) {
		return target;
	}
	if (target.__v_raw && !(isReadonly2 && target['__v_isReactive'])) {
		return target;
	}
	const existingProxy = proxyMap.get(target);
	if (existingProxy) {
		return existingProxy;
	}
	const targetType = getTargetType(target);
	if (targetType === 0) {
		return target;
	}
	const proxy = new Proxy(target, targetType === 2 ? collectionHandlers : baseHandlers);
	proxyMap.set(target, proxy);
	return proxy;
}
function isReactive(value) {
	if (isReadonly(value)) {
		return isReactive(value.__v_raw);
	}
	return !!(value && value['__v_isReactive']);
}
function isReadonly(value) {
	return !!(value && value['__v_isReadonly']);
}
function isShallow(value) {
	return !!(value && value['__v_isShallow']);
}
function isProxy(value) {
	return value ? !!value.__v_raw : false;
}
function toRaw(observed) {
	const raw = observed && observed.__v_raw;
	return raw ? toRaw(raw) : observed;
}
function markRaw(value) {
	if (!hasOwn(value, '__v_skip') && Object.isExtensible(value)) {
		def(value, '__v_skip', true);
	}
	return value;
}
const toReactive = (value) => isObject(value) ? reactive(value) : value;
const toReadonly = (value) => isObject(value) ? readonly(value) : value;
function isRef(r) {
	return r ? r.__v_isRef === true : false;
}
function unref(ref2) {
	return isRef(ref2) ? ref2.value : ref2;
}
const shallowUnwrapHandlers = {
	get: (target, key, receiver) => key === '__v_raw' ? target : unref(Reflect.get(target, key, receiver)),
	set: (target, key, value, receiver) => {
		const oldValue = target[key];
		if (isRef(oldValue) && !isRef(value)) {
			oldValue.value = value;
			return true;
		} else {
			return Reflect.set(target, key, value, receiver);
		}
	}
};
function proxyRefs(objectWithRefs) {
	return isReactive(objectWithRefs) ? objectWithRefs : new Proxy(objectWithRefs, shallowUnwrapHandlers);
}
class ComputedRefImpl {
	constructor(fn, setter, isSSR) {
		this.fn = fn;
		this.setter = setter;
		this._value = undefined;
		this.dep = new Dep(this);
		this.__v_isRef = true;
		this.deps = undefined;
		this.depsTail = undefined;
		this.flags = 16;
		this.globalVersion = globalVersion - 1;
		this.next = undefined;
		this.effect = this;
		this['__v_isReadonly'] = !setter;
		this.isSSR = isSSR;
	}
	notify() {
		this.flags |= 16;
		if (!(this.flags & 8) && false) {
			batch(this, true);
			return true;
		}
	}
	get value() {
		const link = this.dep.track();
		refreshComputed(this);
		if (link) {
			link.version = this.dep.version;
		}
		return this._value;
	}
	set value(newValue) {
		if (this.setter) {
			this.setter(newValue);
		}
	}
}
function computed$1(getterOrOptions, __unused_01FD, isSSR = false) {
	let getter;
	let setter;
	{
		{
			getter = getterOrOptions.get;
			setter = getterOrOptions.set;
		}
	}
	const cRef = new ComputedRefImpl(getter, setter, isSSR);
	return cRef;
}
const INITIAL_WATCHER_VALUE = {};
const cleanupMap = new WeakMap();
let activeWatcher = undefined;
function onWatcherCleanup(cleanupFn, __unused_9AD7, owner) {
	{
		{
			let cleanups = cleanupMap.get(owner);
			if (!cleanups) cleanupMap.set(owner, cleanups = []);
			cleanups.push(cleanupFn);
		}
	}
}
function watch$1(source, cb, options = EMPTY_OBJ) {
	const { immediate, deep, once, scheduler, augmentJob, call } = options;
	const reactiveGetter = (source2) => {
		if (deep) return source2;
		if (isShallow(source2) || deep === false || deep === 0) return traverse(source2, 1);
		return traverse(source2);
	};
	let effect2;
	let getter;
	let cleanup;
	let boundCleanup;
	let forceTrigger = false;
	let isMultiSource = false;
	if (isRef(source)) {
		getter = () => source.value;
		forceTrigger = isShallow(source);
	} else if (isReactive(source)) {
		getter = () => reactiveGetter(source);
		forceTrigger = true;
	} else if (isArray(source)) {
		isMultiSource = true;
		forceTrigger = source.some((s) => isReactive(s) || isShallow(s));
		getter = () => source.map((s) => {
			if (isRef(s)) {
				return s.value;
			} else if (isReactive(s)) {
				return reactiveGetter(s);
			} else if (isFunction(s)) {
				return call ? call(s, 2) : s();
			}
		});
	} else if (isFunction(source)) {
		if (cb) {
			getter = call ? () => call(source, 2) : source;
		} else {
			getter = () => {
				if (cleanup) {
					pauseTracking();
					try {
						cleanup();
					} finally {
						resetTracking();
					}
				}
				const currentEffect = activeWatcher;
				activeWatcher = effect2;
				try {
					return call ? call(source, 3, [boundCleanup]) : source(boundCleanup);
				} finally {
					activeWatcher = currentEffect;
				}
			};
		}
	} else {
		getter = NOOP;
	}
	if (cb && deep) {
		const baseGetter = getter;
		const depth = deep === true ? Infinity : deep;
		getter = () => traverse(baseGetter(), depth);
	}
	const scope = getCurrentScope();
	const watchHandle = () => {
		effect2.stop();
		if (scope) {
			remove(scope.effects, effect2);
		}
	};
	if (once && cb) {
		const _cb = cb;
		cb = (...args) => {
			_cb(...args);
			watchHandle();
		};
	}
	let oldValue = isMultiSource ? new Array(source.length).fill(INITIAL_WATCHER_VALUE) : INITIAL_WATCHER_VALUE;
	const job = (immediateFirstRun) => {
		if (!(effect2.flags & 1) || !effect2.dirty && !immediateFirstRun) {
			return;
		}
		if (cb) {
			const newValue = effect2.run();
			if (deep || forceTrigger || (isMultiSource ? newValue.some((v, i) => hasChanged(v, oldValue[i])) : hasChanged(newValue, oldValue))) {
				const currentWatcher = activeWatcher;
				activeWatcher = effect2;
				try {
					const args = [
						newValue,
						oldValue === INITIAL_WATCHER_VALUE ? undefined : isMultiSource && oldValue[0] === INITIAL_WATCHER_VALUE ? [] : oldValue,
						undefined
					];
					call ? call(cb, 3, args) : cb(...args);
					oldValue = newValue;
				} finally {
					activeWatcher = currentWatcher;
				}
			}
		} else {
			effect2.run();
		}
	};
	if (augmentJob) {
		augmentJob(job);
	}
	effect2 = new ReactiveEffect(getter);
	effect2.scheduler = scheduler ? () => scheduler(job, false) : job;
	boundCleanup = (fn) => (onWatcherCleanup(fn, 0, effect2), undefined);
	cleanup = effect2.onStop = () => {
		const cleanups = cleanupMap.get(effect2);
		if (cleanups) {
			if (call) {
				call(cleanups, 4);
			} else {
				for (const cleanup2 of cleanups) cleanup2();
			}
			cleanupMap.delete(effect2);
		}
	};
	if (cb) {
		if (immediate) {
			job(true);
		} else {
			oldValue = effect2.run();
		}
	} else if (scheduler) {
		scheduler(job.bind(null, true), true);
	} else {
		effect2.run();
	}
	watchHandle.pause = effect2.pause.bind(effect2);
	watchHandle.resume = effect2.resume.bind(effect2);
	watchHandle.stop = watchHandle;
	return watchHandle;
}
function traverse(value, depth = Infinity, seen) {
	if (depth <= 0 || !isObject(value) || value.__v_skip) {
		return value;
	}
	seen = seen || new Set();
	if (seen.has(value)) {
		return value;
	}
	seen.add(value);
	depth--;
	if (isRef(value)) {
		traverse(value.value, depth, seen);
	} else if (isArray(value)) {
		for (let i = 0; i < value.length; i++) {
			traverse(value[i], depth, seen);
		}
	} else if (isSet(value) || isMap(value)) {
		value.forEach((v) => {
			traverse(v, depth, seen);
		});
	} else if (isPlainObject(value)) {
		for (const key in value) {
			traverse(value[key], depth, seen);
		}
		for (const key of Object.getOwnPropertySymbols(value)) {
			if (Object.prototype.propertyIsEnumerable.call(value, key)) {
				traverse(value[key], depth, seen);
			}
		}
	}
	return value;
}
function callWithErrorHandling(fn, instance, type, args) {
	try {
		return args ? fn(...args) : fn();
	} catch (err) {
		handleError(err, instance, type);
	}
}
function callWithAsyncErrorHandling(fn, instance, type, args) {
	if (isFunction(fn)) {
		const res = callWithErrorHandling(fn, instance, type, args);
		if (res && isPromise(res)) {
			res.catch((err) => {
				handleError(err, instance, type);
			});
		}
		return res;
	}
	if (isArray(fn)) {
		const values = [];
		for (let i = 0; i < fn.length; i++) {
			values.push(callWithAsyncErrorHandling(fn[i], instance, type, args));
		}
		return values;
	}
}
function handleError(err, instance, type, throwInDev = true) {
	const __unused_3356 = instance && instance.vnode;
	const { errorHandler, throwUnhandledErrorInProduction } = instance && instance.appContext.config || EMPTY_OBJ;
	if (instance) {
		let cur = instance.parent;
		const exposedInstance = instance.proxy;
		const errorInfo = `https://vuejs.org/error-reference/#runtime-${type}`;
		while (cur) {
			const errorCapturedHooks = cur.ec;
			if (errorCapturedHooks) {
				for (let i = 0; i < errorCapturedHooks.length; i++) {
					if (errorCapturedHooks[i](err, exposedInstance, errorInfo) === false) {
						return;
					}
				}
			}
			cur = cur.parent;
		}
		if (errorHandler) {
			pauseTracking();
			callWithErrorHandling(errorHandler, null, 10, [
				err,
				exposedInstance,
				errorInfo
			]);
			resetTracking();
			return;
		}
	}
	logError(err, 0, 0, 0, throwUnhandledErrorInProduction);
}
function logError(err, __unused_1A06, __unused_0AFD, __unused_35F3, throwInProd = false) {
	if (throwInProd) {
		throw err;
	} else {
		console.error(err);
	}
}
const queue = [];
let flushIndex = -1;
const pendingPostFlushCbs = [];
let activePostFlushCbs = null;
let postFlushIndex = 0;
const resolvedPromise = Promise.resolve();
let currentFlushPromise = null;
function nextTick(fn) {
	const p2 = currentFlushPromise || resolvedPromise;
	return fn ? p2.then(this ? fn.bind(this) : fn) : p2;
}
function findInsertionIndex(id) {
	let start = flushIndex + 1;
	let end = queue.length;
	while (start < end) {
		const middle = start + end >>> 1;
		const middleJob = queue[middle];
		const middleJobId = getId(middleJob);
		if (middleJobId < id || middleJobId === id && middleJob.flags & 2) {
			start = middle + 1;
		} else {
			end = middle;
		}
	}
	return start;
}
function queueJob(job) {
	if (!(job.flags & 1)) {
		const jobId = getId(job);
		const lastJob = queue[queue.length - 1];
		if (!lastJob || !(job.flags & 2) && jobId >= getId(lastJob)) {
			queue.push(job);
		} else {
			queue.splice(findInsertionIndex(jobId), 0, job);
		}
		job.flags |= 1;
		queueFlush();
	}
}
function queueFlush() {
	if (!currentFlushPromise) {
		currentFlushPromise = resolvedPromise.then(flushJobs);
	}
}
function queuePostFlushCb(cb) {
	if (!isArray(cb)) {
		if (activePostFlushCbs && cb.id === -1) {
			activePostFlushCbs.splice(postFlushIndex + 1, 0, cb);
		} else if (!(cb.flags & 1)) {
			pendingPostFlushCbs.push(cb);
			cb.flags |= 1;
		}
	} else {
		pendingPostFlushCbs.push(...cb);
	}
	queueFlush();
}
function flushPreFlushCbs(instance, __unused_40A8, i = flushIndex + 1) {
	for (; i < queue.length; i++) {
		const cb = queue[i];
		if (cb && cb.flags & 2) {
			if (instance && (cb.id, instance.uid, false)) {
				continue;
			}
			queue.splice(i, 1);
			i--;
			if (cb.flags & 4) {
				cb.flags &= ~1;
			}
			cb();
			if (!(cb.flags & 4)) {
				cb.flags &= ~1;
			}
		}
	}
}
function flushPostFlushCbs() {
	if (pendingPostFlushCbs.length) {
		const deduped = [...new Set(pendingPostFlushCbs)].sort((a, b) => getId(a) - getId(b));
		pendingPostFlushCbs.length = 0;
		if (activePostFlushCbs) {
			activePostFlushCbs.push(...deduped);
			return;
		}
		activePostFlushCbs = deduped;
		for (postFlushIndex = 0; postFlushIndex < activePostFlushCbs.length; postFlushIndex++) {
			const cb = activePostFlushCbs[postFlushIndex];
			if (cb.flags & 4) {
				cb.flags &= ~1;
			}
			if (!(cb.flags & 8)) cb();
			cb.flags &= ~1;
		}
		activePostFlushCbs = null;
		postFlushIndex = 0;
	}
}
const getId = (job) => job.id == null ? job.flags & 2 ? -1 : Infinity : job.id;
function flushJobs() {
	try {
		for (flushIndex = 0; flushIndex < queue.length; flushIndex++) {
			const job = queue[flushIndex];
			if (job && !(job.flags & 8)) {
				if (job.flags & 4) {
					job.flags &= ~1;
				}
				callWithErrorHandling(job, job.i, job.i ? 15 : 14);
				if (!(job.flags & 4)) {
					job.flags &= ~1;
				}
			}
		}
	} finally {
		for (; flushIndex < queue.length; flushIndex++) {
			const job = queue[flushIndex];
			if (job) {
				job.flags &= ~1;
			}
		}
		flushIndex = -1;
		queue.length = 0;
		currentFlushPromise = null;
		if (queue.length || 0) {
			flushJobs();
		}
	}
}
let currentRenderingInstance = null;
let currentScopeId = null;
function setCurrentRenderingInstance(instance) {
	const prev = currentRenderingInstance;
	currentRenderingInstance = instance;
	currentScopeId = instance && instance.type.__scopeId || null;
	return prev;
}
function withCtx(fn, ctx = currentRenderingInstance) {
	if (!ctx) return fn;
	if (fn._n) {
		return fn;
	}
	const renderFnWithContext = (...args) => {
		if (renderFnWithContext._d) {
			setBlockTracking(-1);
		}
		const prevInstance = setCurrentRenderingInstance(ctx);
		let res;
		try {
			res = fn(...args);
		} finally {
			setCurrentRenderingInstance(prevInstance);
			if (renderFnWithContext._d) {
				setBlockTracking(1);
			}
		}
		return res;
	};
	renderFnWithContext._n = true;
	renderFnWithContext._c = true;
	renderFnWithContext._d = true;
	return renderFnWithContext;
}
function invokeDirectiveHook(vnode, prevVNode, instance, name) {
	const bindings = vnode.dirs;
	const oldBindings = prevVNode && prevVNode.dirs;
	for (let i = 0; i < bindings.length; i++) {
		const binding = bindings[i];
		if (oldBindings) {
			binding.oldValue = oldBindings[i].value;
		}
		let hook = binding.dir[name];
		if (hook) {
			pauseTracking();
			callWithAsyncErrorHandling(hook, instance, 8, [
				vnode.el,
				binding,
				vnode,
				prevVNode
			]);
			resetTracking();
		}
	}
}
const TeleportEndKey = Symbol('_vte');
const isTeleport = (type) => type.__isTeleport;
function setTransitionHooks(vnode, hooks) {
	if (vnode.shapeFlag & 6 && vnode.component) {
		vnode.transition = hooks;
		setTransitionHooks(vnode.component.subTree, hooks);
	} else if (vnode.shapeFlag & 128) {
		vnode.ssContent.transition = hooks.clone(vnode.ssContent);
		vnode.ssFallback.transition = hooks.clone(vnode.ssFallback);
	} else {
		vnode.transition = hooks;
	}
}
function markAsyncBoundary(instance) {
	instance.ids = [
		instance.ids[0] + instance.ids[2]++ + '-',
		0,
		0
	];
}
function setRef(rawRef, oldRawRef, parentSuspense, vnode, isUnmount = false) {
	if (isArray(rawRef)) {
		rawRef.forEach((r, i) => setRef(r, oldRawRef && (isArray(oldRawRef) ? oldRawRef[i] : oldRawRef), parentSuspense, vnode, isUnmount));
		return;
	}
	if (isAsyncWrapper(vnode) && !isUnmount) {
		return;
	}
	const refValue = vnode.shapeFlag & 4 ? getComponentPublicInstance(vnode.component) : vnode.el;
	const value = isUnmount ? null : refValue;
	const { i: owner, r: ref3 } = rawRef;
	const oldRef = oldRawRef && oldRawRef.r;
	const refs = owner.refs === EMPTY_OBJ ? owner.refs = {} : owner.refs;
	const setupState = owner.setupState;
	const rawSetupState = toRaw(setupState);
	const canSetSetupRef = setupState === EMPTY_OBJ ? () => false : (key) => {
		return hasOwn(rawSetupState, key);
	};
	if (oldRef != null && oldRef !== ref3) {
		if (isString(oldRef)) {
			refs[oldRef] = null;
			if (canSetSetupRef(oldRef)) {
				setupState[oldRef] = null;
			}
		} else if (isRef(oldRef)) {
			oldRef.value = null;
		}
	}
	if (isFunction(ref3)) {
		callWithErrorHandling(ref3, owner, 12, [value, refs]);
	} else {
		const _isString = isString(ref3);
		const _isRef = isRef(ref3);
		if (_isString || _isRef) {
			const doSet = () => {
				if (rawRef.f) {
					const existing = _isString ? canSetSetupRef(ref3) ? setupState[ref3] : refs[ref3] : ref3.value;
					if (isUnmount) {
						isArray(existing) && remove(existing, refValue);
					} else {
						if (!isArray(existing)) {
							if (_isString) {
								refs[ref3] = [refValue];
								if (canSetSetupRef(ref3)) {
									setupState[ref3] = refs[ref3];
								}
							} else {
								ref3.value = [refValue];
								if (rawRef.k) refs[rawRef.k] = ref3.value;
							}
						} else if (!existing.includes(refValue)) {
							existing.push(refValue);
						}
					}
				} else if (_isString) {
					refs[ref3] = value;
					if (canSetSetupRef(ref3)) {
						setupState[ref3] = value;
					}
				} else if (_isRef) {
					ref3.value = value;
					if (rawRef.k) refs[rawRef.k] = value;
				}
			};
			if (value) {
				doSet.id = -1;
				queuePostRenderEffect(doSet, parentSuspense);
			} else {
				doSet();
			}
		}
	}
}
getGlobalThis().requestIdleCallback;
getGlobalThis().cancelIdleCallback;
const isAsyncWrapper = (i) => !!i.type.__asyncLoader;
const isKeepAlive = (vnode) => vnode.type.__isKeepAlive;
function onActivated(hook) {
	registerKeepAliveHook(hook, 'a', undefined);
}
function onDeactivated(hook) {
	registerKeepAliveHook(hook, 'da', undefined);
}
function registerKeepAliveHook(hook, type, target = currentInstance) {
	const wrappedHook = hook.__wdc || (hook.__wdc = () => {
		let current = target;
		while (current) {
			if (current.isDeactivated) {
				return;
			}
			current = current.parent;
		}
		return hook();
	});
	injectHook(type, wrappedHook, target);
	if (target) {
		let current = target.parent;
		while (current && current.parent) {
			if (isKeepAlive(current.parent.vnode)) {
				injectToKeepAliveRoot(wrappedHook, type, target, current);
			}
			current = current.parent;
		}
	}
}
function injectToKeepAliveRoot(hook, type, target, keepAliveRoot) {
	const injected = injectHook(type, hook, keepAliveRoot, true);
	onUnmounted(() => {
		remove(keepAliveRoot[type], injected);
	}, target);
}
function injectHook(type, hook, target = currentInstance, prepend = false) {
	if (target) {
		const hooks = target[type] || (target[type] = []);
		const wrappedHook = hook.__weh || (hook.__weh = (...args) => {
			pauseTracking();
			const reset = setCurrentInstance(target);
			const res = callWithAsyncErrorHandling(hook, target, type, args);
			reset();
			resetTracking();
			return res;
		});
		if (prepend) {
			hooks.unshift(wrappedHook);
		} else {
			hooks.push(wrappedHook);
		}
		return wrappedHook;
	}
}
const createHook = (lifecycle) => (hook, target = currentInstance) => {
	if (!isInSSRComponentSetup || lifecycle === 'sp') {
		injectHook(lifecycle, (...args) => hook(...args), target);
	}
};
const onBeforeMount = createHook('bm');
const onMounted = createHook('m');
const onBeforeUpdate = createHook('bu');
const onUpdated = createHook('u');
const onBeforeUnmount = createHook('bum');
const onUnmounted = createHook('um');
const onServerPrefetch = createHook('sp');
const onRenderTriggered = createHook('rtg');
const onRenderTracked = createHook('rtc');
function onErrorCaptured(hook, target = currentInstance) {
	injectHook('ec', hook, target);
}
const NULL_DYNAMIC_COMPONENT = Symbol.for('v-ndc');
const getPublicInstance = (i) => {
	if (!i) return null;
	if (isStatefulComponent(i)) return getComponentPublicInstance(i);
	return getPublicInstance(i.parent);
};
const publicPropertiesMap = extend(Object.create(null), {
	$: (i) => i,
	$el: (i) => i.vnode.el,
	$data: (i) => i.data,
	$props: (i) => i.props,
	$attrs: (i) => i.attrs,
	$slots: (i) => i.slots,
	$refs: (i) => i.refs,
	$parent: (i) => getPublicInstance(i.parent),
	$root: (i) => getPublicInstance(i.root),
	$host: (i) => i.ce,
	$emit: (i) => i.emit,
	$options: (i) => resolveMergedOptions(i),
	$forceUpdate: (i) => i.f || (i.f = () => {
		queueJob(i.update);
	}),
	$nextTick: (i) => i.n || (i.n = nextTick.bind(i.proxy)),
	$watch: (i) => instanceWatch.bind(i)
});
const hasSetupBinding = (state, key) => state !== EMPTY_OBJ && !state.__isScriptSetup && hasOwn(state, key);
const PublicInstanceProxyHandlers = {
	get({ _: instance }, key) {
		if (key === '__v_skip') {
			return true;
		}
		const { ctx, setupState, data, props, accessCache, type, appContext } = instance;
		let normalizedProps;
		if (key[0] !== '$') {
			const n = accessCache[key];
			if (n !== undefined) {
				switch (n) {
					case 1: return setupState[key];
					case 2: return data[key];
					case 4: return ctx[key];
					case 3: return props[key];
				}
			} else if (hasSetupBinding(setupState, key)) {
				accessCache[key] = 1;
				return setupState[key];
			} else if (data !== EMPTY_OBJ && hasOwn(data, key)) {
				accessCache[key] = 2;
				return data[key];
			} else if ((normalizedProps = instance.propsOptions[0]) && hasOwn(normalizedProps, key)) {
				accessCache[key] = 3;
				return props[key];
			} else if (ctx !== EMPTY_OBJ && hasOwn(ctx, key)) {
				accessCache[key] = 4;
				return ctx[key];
			} else if (shouldCacheAccess) {
				accessCache[key] = 0;
			}
		}
		const publicGetter = publicPropertiesMap[key];
		let cssModule, globalProperties;
		if (publicGetter) {
			if (key === '$attrs') {
				track(instance.attrs, 0, '');
			}
			return publicGetter(instance);
		} else if ((cssModule = type.__cssModules) && (cssModule = cssModule[key])) {
			return cssModule;
		} else if (ctx !== EMPTY_OBJ && hasOwn(ctx, key)) {
			accessCache[key] = 4;
			return ctx[key];
		} else if (globalProperties = appContext.config.globalProperties, hasOwn(globalProperties, key)) {
			{
				return globalProperties[key];
			}
		}
	},
	set({ _: instance }, key, value) {
		const { data, setupState, ctx } = instance;
		if (hasSetupBinding(setupState, key)) {
			setupState[key] = value;
			return true;
		} else if (data !== EMPTY_OBJ && hasOwn(data, key)) {
			data[key] = value;
			return true;
		} else if (hasOwn(instance.props, key)) {
			return false;
		}
		if (key[0] === '$' && key.slice(1) in instance) {
			return false;
		} else {
			{
				ctx[key] = value;
			}
		}
		return true;
	},
	has({ _: { data, setupState, accessCache, ctx, appContext, propsOptions } }, key) {
		let normalizedProps;
		return !!accessCache[key] || data !== EMPTY_OBJ && hasOwn(data, key) || hasSetupBinding(setupState, key) || (normalizedProps = propsOptions[0]) && hasOwn(normalizedProps, key) || hasOwn(ctx, key) || hasOwn(publicPropertiesMap, key) || hasOwn(appContext.config.globalProperties, key);
	},
	defineProperty(target, key, descriptor) {
		if (descriptor.get != null) {
			target._.accessCache[key] = 0;
		} else if (hasOwn(descriptor, 'value')) {
			this.set(target, key, descriptor.value, null);
		}
		return Reflect.defineProperty(target, key, descriptor);
	}
};
function normalizePropsOrEmits(props) {
	return isArray(props) ? props.reduce((normalized, p2) => (normalized[p2] = null, normalized), {}) : props;
}
let shouldCacheAccess = true;
function applyOptions(instance) {
	const options = resolveMergedOptions(instance);
	const publicThis = instance.proxy;
	const ctx = instance.ctx;
	shouldCacheAccess = false;
	if (options.beforeCreate) {
		callHook(options.beforeCreate, instance, 'bc');
	}
	const { data: dataOptions, computed: computedOptions, methods, watch: watchOptions, provide: provideOptions, inject: injectOptions, created, beforeMount, mounted, beforeUpdate, updated, activated, deactivated, beforeDestroy, beforeUnmount, destroyed, unmounted, render, renderTracked, renderTriggered, errorCaptured, serverPrefetch, expose, inheritAttrs, components, directives, filters } = options;
	if (injectOptions) {
		resolveInjections(injectOptions, ctx);
	}
	if (methods) {
		for (const key in methods) {
			const methodHandler = methods[key];
			if (isFunction(methodHandler)) {
				{
					ctx[key] = methodHandler.bind(publicThis);
				}
			}
		}
	}
	if (dataOptions) {
		const data = dataOptions.call(publicThis, publicThis);
		if (!!isObject(data)) {
			instance.data = reactive(data);
		}
	}
	shouldCacheAccess = true;
	if (computedOptions) {
		for (const key in computedOptions) {
			const opt = computedOptions[key];
			const get = isFunction(opt) ? opt.bind(publicThis, publicThis) : isFunction(opt.get) ? opt.get.bind(publicThis, publicThis) : NOOP;
			const set = !isFunction(opt) && isFunction(opt.set) ? opt.set.bind(publicThis) : NOOP;
			const c = computed({
				get,
				set
			});
			Object.defineProperty(ctx, key, {
				enumerable: true,
				configurable: true,
				get: () => c.value,
				set: (v) => c.value = v
			});
		}
	}
	if (watchOptions) {
		for (const key in watchOptions) {
			createWatcher(watchOptions[key], ctx, publicThis, key);
		}
	}
	if (provideOptions) {
		const provides = isFunction(provideOptions) ? provideOptions.call(publicThis) : provideOptions;
		Reflect.ownKeys(provides).forEach((key) => {
			provide(key, provides[key]);
		});
	}
	if (created) {
		callHook(created, instance, 'c');
	}
	function registerLifecycleHook(register, hook) {
		if (isArray(hook)) {
			hook.forEach((_hook) => (register(_hook.bind(publicThis)), undefined));
		} else if (hook) {
			register(hook.bind(publicThis));
		}
	}
	registerLifecycleHook(onBeforeMount, beforeMount);
	registerLifecycleHook(onMounted, mounted);
	registerLifecycleHook(onBeforeUpdate, beforeUpdate);
	registerLifecycleHook(onUpdated, updated);
	registerLifecycleHook(onActivated, activated);
	registerLifecycleHook(onDeactivated, deactivated);
	registerLifecycleHook(onErrorCaptured, errorCaptured);
	registerLifecycleHook(onRenderTracked, renderTracked);
	registerLifecycleHook(onRenderTriggered, renderTriggered);
	registerLifecycleHook(onBeforeUnmount, beforeUnmount);
	registerLifecycleHook(onUnmounted, unmounted);
	registerLifecycleHook(onServerPrefetch, serverPrefetch);
	if (isArray(expose)) {
		if (expose.length) {
			const exposed = instance.exposed || (instance.exposed = {});
			expose.forEach((key) => {
				Object.defineProperty(exposed, key, {
					get: () => publicThis[key],
					set: (val) => publicThis[key] = val
				});
			});
		} else if (!instance.exposed) {
			instance.exposed = {};
		}
	}
	if (render && instance.render === NOOP) {
		instance.render = render;
	}
	if (inheritAttrs != null) {
		instance.inheritAttrs = inheritAttrs;
	}
	if (components) instance.components = components;
	if (directives) instance.directives = directives;
	if (serverPrefetch) {
		markAsyncBoundary(instance);
	}
}
function resolveInjections(injectOptions, ctx) {
	if (isArray(injectOptions)) {
		injectOptions = normalizeInject(injectOptions);
	}
	for (const key in injectOptions) {
		const opt = injectOptions[key];
		let injected;
		if (isObject(opt)) {
			if ('default' in opt) {
				injected = inject(opt.from || key, opt.default, true);
			} else {
				injected = inject(opt.from || key);
			}
		} else {
			injected = inject(opt);
		}
		if (isRef(injected)) {
			Object.defineProperty(ctx, key, {
				enumerable: true,
				configurable: true,
				get: () => injected.value,
				set: (v) => injected.value = v
			});
		} else {
			ctx[key] = injected;
		}
	}
}
function callHook(hook, instance, type) {
	callWithAsyncErrorHandling(isArray(hook) ? hook.map((h2) => h2.bind(instance.proxy)) : hook.bind(instance.proxy), instance, type);
}
function createWatcher(raw, ctx, publicThis, key) {
	let getter = key.includes('.') ? createPathGetter(publicThis, key) : () => publicThis[key];
	if (isString(raw)) {
		const handler = ctx[raw];
		if (isFunction(handler)) {
			{
				watch(getter, handler);
			}
		}
	} else if (isFunction(raw)) {
		{
			watch(getter, raw.bind(publicThis));
		}
	} else if (isObject(raw)) {
		if (isArray(raw)) {
			raw.forEach((r) => createWatcher(r, ctx, publicThis, key));
		} else {
			const handler = isFunction(raw.handler) ? raw.handler.bind(publicThis) : ctx[raw.handler];
			if (isFunction(handler)) {
				watch(getter, handler, raw);
			}
		}
	}
}
function resolveMergedOptions(instance) {
	const base = instance.type;
	const { mixins, extends: extendsOptions } = base;
	const { mixins: globalMixins, optionsCache: cache, config: { optionMergeStrategies } } = instance.appContext;
	const cached = cache.get(base);
	let resolved;
	if (cached) {
		resolved = cached;
	} else if (!globalMixins.length && !mixins && !extendsOptions) {
		{
			resolved = base;
		}
	} else {
		resolved = {};
		if (globalMixins.length) {
			globalMixins.forEach((m) => mergeOptions(resolved, m, optionMergeStrategies, true));
		}
		mergeOptions(resolved, base, optionMergeStrategies);
	}
	if (isObject(base)) {
		cache.set(base, resolved);
	}
	return resolved;
}
function mergeOptions(to, from, strats, asMixin = false) {
	const { mixins, extends: extendsOptions } = from;
	if (extendsOptions) {
		mergeOptions(to, extendsOptions, strats, true);
	}
	if (mixins) {
		mixins.forEach((m) => mergeOptions(to, m, strats, true));
	}
	for (const key in from) {
		if (!(asMixin && key === 'expose')) {
			const strat = internalOptionMergeStrats[key] || strats && strats[key];
			to[key] = strat ? strat(to[key], from[key]) : from[key];
		}
	}
	return to;
}
const internalOptionMergeStrats = {
	data: mergeDataFn,
	props: mergeEmitsOrPropsOptions,
	emits: mergeEmitsOrPropsOptions,
	methods: mergeObjectOptions,
	computed: mergeObjectOptions,
	beforeCreate: mergeAsArray,
	created: mergeAsArray,
	beforeMount: mergeAsArray,
	mounted: mergeAsArray,
	beforeUpdate: mergeAsArray,
	updated: mergeAsArray,
	beforeDestroy: mergeAsArray,
	beforeUnmount: mergeAsArray,
	destroyed: mergeAsArray,
	unmounted: mergeAsArray,
	activated: mergeAsArray,
	deactivated: mergeAsArray,
	errorCaptured: mergeAsArray,
	serverPrefetch: mergeAsArray,
	components: mergeObjectOptions,
	directives: mergeObjectOptions,
	watch: mergeWatchOptions,
	provide: mergeDataFn,
	inject: mergeInject
};
function mergeDataFn(to, from) {
	if (!from) {
		return to;
	}
	if (!to) {
		return from;
	}
	return function mergedDataFn() {
		return extend(isFunction(to) ? to.call(this, this) : to, isFunction(from) ? from.call(this, this) : from);
	};
}
function mergeInject(to, from) {
	return mergeObjectOptions(normalizeInject(to), normalizeInject(from));
}
function normalizeInject(raw) {
	if (isArray(raw)) {
		const res = {};
		for (let i = 0; i < raw.length; i++) {
			res[raw[i]] = raw[i];
		}
		return res;
	}
	return raw;
}
function mergeAsArray(to, from) {
	return to ? [...new Set([].concat(to, from))] : from;
}
function mergeObjectOptions(to, from) {
	return to ? extend(Object.create(null), to, from) : from;
}
function mergeEmitsOrPropsOptions(to, from) {
	if (to) {
		if (isArray(to) && isArray(from)) {
			return [...new Set([...to, ...from])];
		}
		return extend(Object.create(null), normalizePropsOrEmits(to), normalizePropsOrEmits(from != null ? from : {}));
	} else {
		return from;
	}
}
function mergeWatchOptions(to, from) {
	if (!to) return from;
	if (!from) return to;
	const merged = extend(Object.create(null), to);
	for (const key in from) {
		merged[key] = mergeAsArray(to[key], from[key]);
	}
	return merged;
}
function createAppContext() {
	return {
		app: null,
		config: {
			isNativeTag: NO,
			performance: false,
			globalProperties: {},
			optionMergeStrategies: {},
			errorHandler: undefined,
			warnHandler: undefined,
			compilerOptions: {}
		},
		mixins: [],
		components: {},
		directives: {},
		provides: Object.create(null),
		optionsCache: new WeakMap(),
		propsCache: new WeakMap(),
		emitsCache: new WeakMap()
	};
}
function createAppAPI(render) {
	return function createApp2(rootComponent, rootProps) {
		{
			{
				rootComponent = extend({}, rootComponent);
			}
		}
		if (rootProps != null && !isObject(rootProps)) {
			rootProps = null;
		}
		const context = createAppContext();
		const installedPlugins = new WeakSet();
		const pluginCleanupFns = [];
		let isMounted = false;
		const app = context.app = {
			_uid: 0,
			_component: rootComponent,
			_props: rootProps,
			_container: null,
			_context: context,
			_instance: null,
			version: '3.5.12',
			get config() {
				return context.config;
			},
			set config(__unused_D8AF) {},
			use(plugin, ...options) {
				if (!installedPlugins.has(plugin)) {
					if (plugin && isFunction(plugin.install)) {
						installedPlugins.add(plugin);
						plugin.install(app, ...options);
					} else if (isFunction(plugin)) {
						installedPlugins.add(plugin);
						plugin(app, ...options);
					}
				}
				return app;
			},
			mixin(mixin) {
				{
					if (!context.mixins.includes(mixin)) {
						context.mixins.push(mixin);
					}
				}
				return app;
			},
			component(name, component) {
				if (!component) {
					return context.components[name];
				}
				context.components[name] = component;
				return app;
			},
			directive(name, directive) {
				if (!directive) {
					return context.directives[name];
				}
				context.directives[name] = directive;
				return app;
			},
			mount(rootContainer, isHydrate, namespace) {
				if (!isMounted) {
					const vnode = app._ceVNode || createVNode(rootComponent, rootProps);
					vnode.appContext = context;
					if (namespace === true) {
						namespace = 'svg';
					} else if (namespace === false) {
						namespace = undefined;
					}
					if (isHydrate && undefined) {
						undefined(vnode, rootContainer);
					} else {
						render(vnode, rootContainer, namespace);
					}
					isMounted = true;
					app._container = rootContainer;
					rootContainer.__vue_app__ = app;
					return getComponentPublicInstance(vnode.component);
				}
			},
			onUnmount(cleanupFn) {
				pluginCleanupFns.push(cleanupFn);
			},
			unmount() {
				if (isMounted) {
					callWithAsyncErrorHandling(pluginCleanupFns, app._instance, 16);
					render(null, app._container);
					delete app._container.__vue_app__;
				}
			},
			provide(key, value) {
				context.provides[key] = value;
				return app;
			},
			runWithContext(fn) {
				const lastApp = currentApp;
				currentApp = app;
				try {
					return fn();
				} finally {
					currentApp = lastApp;
				}
			}
		};
		return app;
	};
}
let currentApp = null;
function provide(key, value) {
	if (!!currentInstance) {
		let provides = currentInstance.provides;
		const parentProvides = currentInstance.parent && currentInstance.parent.provides;
		if (parentProvides === provides) {
			provides = currentInstance.provides = Object.create(parentProvides);
		}
		provides[key] = value;
	}
}
function inject(key, defaultValue, treatDefaultAsFactory = false) {
	const instance = currentInstance || currentRenderingInstance;
	if (instance || currentApp) {
		const provides = currentApp ? currentApp._context.provides : instance ? instance.parent == null ? instance.vnode.appContext && instance.vnode.appContext.provides : instance.parent.provides : undefined;
		if (provides && key in provides) {
			return provides[key];
		} else if (arguments.length > 1) {
			return treatDefaultAsFactory && isFunction(defaultValue) ? defaultValue.call(instance && instance.proxy) : defaultValue;
		}
	}
}
const internalObjectProto = {};
const createInternalObject = () => Object.create(internalObjectProto);
const isInternalObject = (obj) => Object.getPrototypeOf(obj) === internalObjectProto;
function initProps(instance, rawProps, isStateful) {
	const props = {};
	const attrs = createInternalObject();
	instance.propsDefaults = Object.create(null);
	setFullProps(instance, rawProps, props, attrs);
	for (const key in instance.propsOptions[0]) {
		if (!(key in props)) {
			props[key] = undefined;
		}
	}
	if (isStateful) {
		instance.props = shallowReactive(props);
	} else {
		if (!instance.type.props) {
			instance.props = attrs;
		} else {
			instance.props = props;
		}
	}
	instance.attrs = attrs;
}
function updateProps(instance, rawProps, rawPrevProps, optimized) {
	const { props, attrs, vnode: { patchFlag } } = instance;
	const rawCurrentProps = toRaw(props);
	const [options] = instance.propsOptions;
	let hasAttrsChanged = false;
	if ((optimized || patchFlag > 0) && !(patchFlag & 16)) {
		if (patchFlag & 8) {
			const propsToUpdate = instance.vnode.dynamicProps;
			for (let i = 0; i < propsToUpdate.length; i++) {
				let key = propsToUpdate[i];
				if (isEmitListener(instance.emitsOptions, key)) {
					continue;
				}
				const value = rawProps[key];
				if (options) {
					if (hasOwn(attrs, key)) {
						if (value !== attrs[key]) {
							attrs[key] = value;
							hasAttrsChanged = true;
						}
					} else {
						const camelizedKey = camelize(key);
						props[camelizedKey] = resolvePropValue(options, 0, camelizedKey, value, instance, false);
					}
				} else {
					if (value !== attrs[key]) {
						attrs[key] = value;
						hasAttrsChanged = true;
					}
				}
			}
		}
	} else {
		if (setFullProps(instance, rawProps, props, attrs)) {
			hasAttrsChanged = true;
		}
		let kebabKey;
		for (const key in rawCurrentProps) {
			if (!rawProps || !hasOwn(rawProps, key) && ((kebabKey = hyphenate(key)) === key || !hasOwn(rawProps, kebabKey))) {
				if (options) {
					if (rawPrevProps && (rawPrevProps[key] !== undefined || rawPrevProps[kebabKey] !== undefined)) {
						props[key] = resolvePropValue(options, 0, key, undefined, instance);
					}
				} else {
					delete props[key];
				}
			}
		}
		if (attrs !== rawCurrentProps) {
			for (const key in attrs) {
				if (!rawProps || !hasOwn(rawProps, key) && true) {
					delete attrs[key];
					hasAttrsChanged = true;
				}
			}
		}
	}
	if (hasAttrsChanged) {
		trigger(instance.attrs, 'set', '');
	}
}
function setFullProps(instance, rawProps, props, attrs) {
	const [options, needCastKeys] = instance.propsOptions;
	let hasAttrsChanged = false;
	let rawCastValues;
	if (rawProps) {
		for (let key in rawProps) {
			if (isReservedProp(key)) {
				continue;
			}
			const value = rawProps[key];
			let camelKey;
			if (options && hasOwn(options, camelKey = camelize(key))) {
				if (!needCastKeys || !needCastKeys.includes(camelKey)) {
					props[camelKey] = value;
				} else {
					(rawCastValues || (rawCastValues = {}))[camelKey] = value;
				}
			} else if (!isEmitListener(instance.emitsOptions, key)) {
				if (!(key in attrs) || value !== attrs[key]) {
					attrs[key] = value;
					hasAttrsChanged = true;
				}
			}
		}
	}
	if (needCastKeys) {
		const __unused_F43E = toRaw(props);
		const castValues = rawCastValues || EMPTY_OBJ;
		for (let i = 0; i < needCastKeys.length; i++) {
			const key = needCastKeys[i];
			props[key] = resolvePropValue(options, 0, key, castValues[key], instance, !hasOwn(castValues, key));
		}
	}
	return hasAttrsChanged;
}
function resolvePropValue(options, __unused_D326, key, value, instance, isAbsent) {
	const opt = options[key];
	if (opt != null) {
		const hasDefault = hasOwn(opt, 'default');
		if (hasDefault && value === undefined) {
			const defaultValue = opt.default;
			{
				opt.type, Function;
				{
					value = defaultValue;
				}
			}
			if (instance.ce) {
				instance.ce._setProp(key, value);
			}
		}
		if (opt[0]) {
			if (isAbsent && !hasDefault) {
				value = false;
			} else if (opt[1] && (value === '' || value === hyphenate(key))) {
				value = true;
			}
		}
	}
	return value;
}
const mixinPropsCache = new WeakMap();
function normalizePropsOptions(comp, appContext, asMixin = false) {
	const cache = asMixin ? mixinPropsCache : appContext.propsCache;
	const cached = cache.get(comp);
	if (cached) {
		return cached;
	}
	const raw = comp.props;
	const normalized = {};
	const needCastKeys = [];
	let hasExtends = false;
	if (!isFunction(comp)) {
		const extendProps = (raw2) => {
			hasExtends = true;
			const [props, keys] = normalizePropsOptions(raw2, appContext, true);
			extend(normalized, props);
			if (keys) needCastKeys.push(...keys);
		};
		if (!asMixin && appContext.mixins.length) {
			appContext.mixins.forEach(extendProps);
		}
		if (comp.extends) {
			extendProps(comp.extends);
		}
		if (comp.mixins) {
			comp.mixins.forEach(extendProps);
		}
	}
	if (!raw && !hasExtends) {
		if (isObject(comp)) {
			cache.set(comp, EMPTY_ARR);
		}
		return EMPTY_ARR;
	}
	if (isArray(raw)) {
		for (let i = 0; i < raw.length; i++) {
			const normalizedKey = camelize(raw[i]);
			if (validatePropName(normalizedKey)) {
				normalized[normalizedKey] = EMPTY_OBJ;
			}
		}
	} else if (raw) {
		for (const key in raw) {
			const normalizedKey = camelize(key);
			if (validatePropName(normalizedKey)) {
				const opt = raw[key];
				const prop = normalized[normalizedKey] = isArray(opt) || isFunction(opt) ? { type: opt } : extend({}, opt);
				const propType = prop.type;
				let shouldCast = false;
				let shouldCastTrue = true;
				if (isArray(propType)) {
					for (let index = 0; index < propType.length; ++index) {
						const type = propType[index];
						const typeName = isFunction(type) && type.name;
						if (typeName === 'Boolean') {
							shouldCast = true;
							break;
						} else if (typeName === 'String') {
							shouldCastTrue = false;
						}
					}
				} else {
					shouldCast = isFunction(propType) && propType.name === 'Boolean';
				}
				prop[0] = shouldCast;
				prop[1] = shouldCastTrue;
				if (shouldCast || hasOwn(prop, 'default')) {
					needCastKeys.push(normalizedKey);
				}
			}
		}
	}
	const res = [normalized, needCastKeys];
	if (isObject(comp)) {
		cache.set(comp, res);
	}
	return res;
}
function validatePropName(key) {
	if (key[0] !== '$' && !isReservedProp(key)) {
		return true;
	}
	return false;
}
const isInternalKey = (key) => key[0] === '_' || key === '$stable';
const normalizeSlotValue = (value) => isArray(value) ? value.map(normalizeVNode) : [normalizeVNode(value)];
const normalizeSlot = (__unused_0362, rawSlot, ctx) => {
	if (rawSlot._n) {
		return rawSlot;
	}
	const normalized = withCtx((...args) => {
		return normalizeSlotValue(rawSlot(...args));
	}, ctx);
	normalized._c = false;
	return normalized;
};
const normalizeObjectSlots = (rawSlots, slots) => {
	const ctx = rawSlots._ctx;
	for (const key in rawSlots) {
		if (isInternalKey(key)) continue;
		const value = rawSlots[key];
		if (isFunction(value)) {
			slots[key] = normalizeSlot(0, value, ctx);
		} else if (value != null) {
			const normalized = normalizeSlotValue(value);
			slots[key] = () => normalized;
		}
	}
};
const normalizeVNodeSlots = (instance, children) => {
	const normalized = normalizeSlotValue(children);
	instance.slots.default = () => normalized;
};
const assignSlots = (slots, children, optimized) => {
	for (const key in children) {
		if (optimized || key !== '_') {
			slots[key] = children[key];
		}
	}
};
const initSlots = (instance, children, optimized) => {
	const slots = instance.slots = createInternalObject();
	if (instance.vnode.shapeFlag & 32) {
		const type = children._;
		if (type) {
			assignSlots(slots, children, optimized);
			if (optimized) {
				def(slots, '_', type, true);
			}
		} else {
			normalizeObjectSlots(children, slots);
		}
	} else if (children) {
		normalizeVNodeSlots(instance, children);
	}
};
const updateSlots = (instance, children, optimized) => {
	const { vnode, slots } = instance;
	let needDeletionCheck = true;
	let deletionComparisonTarget = EMPTY_OBJ;
	if (vnode.shapeFlag & 32) {
		const type = children._;
		if (type) {
			if (optimized && type === 1) {
				needDeletionCheck = false;
			} else {
				assignSlots(slots, children, optimized);
			}
		} else {
			needDeletionCheck = !children.$stable;
			normalizeObjectSlots(children, slots);
		}
		deletionComparisonTarget = children;
	} else if (children) {
		normalizeVNodeSlots(instance, children);
		deletionComparisonTarget = { default: 1 };
	}
	if (needDeletionCheck) {
		for (const key in slots) {
			if (!isInternalKey(key) && deletionComparisonTarget[key] == null) {
				delete slots[key];
			}
		}
	}
};
const queuePostRenderEffect = queueEffectWithSuspense;
function createRenderer(options) {
	return baseCreateRenderer(options);
}
function baseCreateRenderer(options) {
	const target = getGlobalThis();
	target.__VUE__ = true;
	const { insert: hostInsert, remove: hostRemove, patchProp: hostPatchProp, createElement: hostCreateElement, createText: hostCreateText, createComment: hostCreateComment, setText: hostSetText, setElementText: hostSetElementText, parentNode: hostParentNode, nextSibling: hostNextSibling, setScopeId: hostSetScopeId = NOOP, insertStaticContent: hostInsertStaticContent } = options;
	const patch = (n1, n2, container, anchor = null, parentComponent = null, parentSuspense = null, namespace = undefined, slotScopeIds = null, optimized = !!n2.dynamicChildren) => {
		if (n1 === n2) {
			return;
		}
		if (n1 && (isSameVNodeType(n1, n2), false)) {
			anchor = getNextHostNode(n1);
			unmount(n1, parentComponent, parentSuspense, true);
			n1 = null;
		}
		if (n2.patchFlag === -2) {
			optimized = false;
			n2.dynamicChildren = null;
		}
		const { type, ref: ref3, shapeFlag } = n2;
		switch (type) {
			case Text:
				processText(n1, n2, container, anchor);
				break;
			case Comment:
				processCommentNode(n1, n2, container, anchor);
				break;
			case Static:
				if (n1 == null) {
					mountStaticNode(n2, container, anchor, namespace);
				}
				break;
			case Fragment:
				processFragment(n1, n2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
				break;
			default: if (shapeFlag & 1) {
				processElement(n1, n2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
			} else if (shapeFlag & 6) {
				processComponent(n1, n2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
			} else if (shapeFlag & 64) {
				type.process(n1, n2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized, internals);
			} else if (shapeFlag & 128) {
				type.process(n1, n2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized, internals);
			}
		}
		if (ref3 != null && parentComponent) {
			setRef(ref3, n1 && n1.ref, parentSuspense, n2 || n1, !n2);
		}
	};
	const processText = (n1, n2, container, anchor) => {
		if (n1 == null) {
			hostInsert(n2.el = hostCreateText(n2.children), container, anchor);
		} else {
			const el = n2.el = n1.el;
			if (n2.children !== n1.children) {
				hostSetText(el, n2.children);
			}
		}
	};
	const processCommentNode = (n1, n2, container, anchor) => {
		if (n1 == null) {
			hostInsert(n2.el = hostCreateComment(n2.children || ''), container, anchor);
		} else {
			n2.el = n1.el;
		}
	};
	const mountStaticNode = (n2, container, anchor, namespace) => {
		[n2.el, n2.anchor] = hostInsertStaticContent(n2.children, container, anchor, namespace, n2.el, n2.anchor);
	};
	const moveStaticNode = ({ el, anchor }, container, nextSibling) => {
		let next;
		while (el && el !== anchor) {
			next = hostNextSibling(el);
			hostInsert(el, container, nextSibling);
			el = next;
		}
		hostInsert(anchor, container, nextSibling);
	};
	const removeStaticNode = ({ el, anchor }) => {
		let next;
		while (el && el !== anchor) {
			next = hostNextSibling(el);
			hostRemove(el);
			el = next;
		}
		hostRemove(anchor);
	};
	const processElement = (n1, n2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized) => {
		if (n2.type === 'svg') {
			namespace = 'svg';
		} else if (n2.type === 'math') {
			namespace = 'mathml';
		}
		if (n1 == null) {
			mountElement(n2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
		} else {
			patchElement(n1, n2, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
		}
	};
	const mountElement = (vnode, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized) => {
		let el;
		let vnodeHook;
		const { props, shapeFlag, transition, dirs } = vnode;
		el = vnode.el = hostCreateElement(vnode.type, namespace, props && props.is, props);
		if (shapeFlag & 8) {
			hostSetElementText(el, vnode.children);
		} else if (shapeFlag & 16) {
			mountChildren(vnode.children, el, null, parentComponent, parentSuspense, resolveChildrenNamespace(vnode, namespace), slotScopeIds, optimized);
		}
		if (dirs) {
			invokeDirectiveHook(vnode, null, parentComponent, 'created');
		}
		setScopeId(el, vnode, vnode.scopeId, slotScopeIds, parentComponent);
		if (props) {
			for (const key in props) {
				if (key !== 'value' && !isReservedProp(key)) {
					hostPatchProp(el, key, null, props[key], namespace, parentComponent);
				}
			}
			if ('value' in props) {
				hostPatchProp(el, 'value', null, props.value, namespace);
			}
			if (vnodeHook = props.onVnodeBeforeMount) {
				invokeVNodeHook(vnodeHook, parentComponent, vnode);
			}
		}
		if (dirs) {
			invokeDirectiveHook(vnode, null, parentComponent, 'beforeMount');
		}
		const needCallTransitionHooks = needTransition(parentSuspense, transition);
		if (needCallTransitionHooks) {
			transition.beforeEnter(el);
		}
		hostInsert(el, container, anchor);
		if ((vnodeHook = props && props.onVnodeMounted) || needCallTransitionHooks || dirs) {
			queuePostRenderEffect(() => {
				vnodeHook && invokeVNodeHook(vnodeHook, parentComponent, vnode);
				needCallTransitionHooks && transition.enter(el);
				dirs && invokeDirectiveHook(vnode, null, parentComponent, 'mounted');
			}, parentSuspense);
		}
	};
	const setScopeId = (el, vnode, scopeId, slotScopeIds, parentComponent) => {
		if (scopeId) {
			hostSetScopeId(el, scopeId);
		}
		if (slotScopeIds) {
			for (let i = 0; i < slotScopeIds.length; i++) {
				hostSetScopeId(el, slotScopeIds[i]);
			}
		}
		if (parentComponent) {
			let subTree = parentComponent.subTree;
			if (vnode === subTree || isSuspense(subTree.type) && (subTree.ssContent === vnode || subTree.ssFallback === vnode)) {
				const parentVNode = parentComponent.vnode;
				setScopeId(el, parentVNode, parentVNode.scopeId, parentVNode.slotScopeIds, parentComponent.parent);
			}
		}
	};
	const mountChildren = (children, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized, start = 0) => {
		for (let i = start; i < children.length; i++) {
			const child = children[i] = optimized ? cloneIfMounted(children[i]) : normalizeVNode(children[i]);
			patch(null, child, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
		}
	};
	const patchElement = (n1, n2, parentComponent, parentSuspense, namespace, slotScopeIds, optimized) => {
		const el = n2.el = n1.el;
		let { patchFlag, dynamicChildren, dirs } = n2;
		patchFlag |= n1.patchFlag & 16;
		const oldProps = n1.props || EMPTY_OBJ;
		const newProps = n2.props || EMPTY_OBJ;
		let vnodeHook;
		parentComponent && toggleRecurse(parentComponent, false);
		if (vnodeHook = newProps.onVnodeBeforeUpdate) {
			invokeVNodeHook(vnodeHook, parentComponent, n2, n1);
		}
		if (dirs) {
			invokeDirectiveHook(n2, n1, parentComponent, 'beforeUpdate');
		}
		parentComponent && toggleRecurse(parentComponent, true);
		if (oldProps.innerHTML && newProps.innerHTML == null || oldProps.textContent && newProps.textContent == null) {
			hostSetElementText(el, '');
		}
		if (dynamicChildren) {
			patchBlockChildren(n1.dynamicChildren, dynamicChildren, el, parentComponent, parentSuspense, resolveChildrenNamespace(n2, namespace), slotScopeIds);
		} else if (!optimized) {
			patchChildren(n1, n2, el, null, parentComponent, parentSuspense, resolveChildrenNamespace(n2, namespace), slotScopeIds, false);
		}
		if (patchFlag > 0) {
			if (patchFlag & 16) {
				patchProps(el, oldProps, newProps, parentComponent, namespace);
			} else {
				if (patchFlag & 2) {
					if (oldProps.class !== newProps.class) {
						hostPatchProp(el, 'class', null, newProps.class, namespace);
					}
				}
				if (patchFlag & 4) {
					hostPatchProp(el, 'style', oldProps.style, newProps.style, namespace);
				}
				if (patchFlag & 8) {
					const propsToUpdate = n2.dynamicProps;
					for (let i = 0; i < propsToUpdate.length; i++) {
						const key = propsToUpdate[i];
						const prev = oldProps[key];
						const next = newProps[key];
						if (next !== prev || key === 'value') {
							hostPatchProp(el, key, prev, next, namespace, parentComponent);
						}
					}
				}
			}
			if (patchFlag & 1) {
				if (n1.children !== n2.children) {
					hostSetElementText(el, n2.children);
				}
			}
		} else if (!optimized && dynamicChildren == null) {
			patchProps(el, oldProps, newProps, parentComponent, namespace);
		}
		if ((vnodeHook = newProps.onVnodeUpdated) || dirs) {
			queuePostRenderEffect(() => {
				vnodeHook && invokeVNodeHook(vnodeHook, parentComponent, n2, n1);
				dirs && invokeDirectiveHook(n2, n1, parentComponent, 'updated');
			}, parentSuspense);
		}
	};
	const patchBlockChildren = (oldChildren, newChildren, fallbackContainer, parentComponent, parentSuspense, namespace, slotScopeIds) => {
		for (let i = 0; i < newChildren.length; i++) {
			const oldVNode = oldChildren[i];
			const newVNode = newChildren[i];
			const container = oldVNode.el && (oldVNode.type === Fragment || (isSameVNodeType(oldVNode, newVNode), false) || oldVNode.shapeFlag & (6 | 64)) ? hostParentNode(oldVNode.el) : fallbackContainer;
			patch(oldVNode, newVNode, container, null, parentComponent, parentSuspense, namespace, slotScopeIds, true);
		}
	};
	const patchProps = (el, oldProps, newProps, parentComponent, namespace) => {
		if (oldProps !== newProps) {
			if (oldProps !== EMPTY_OBJ) {
				for (const key in oldProps) {
					if (!isReservedProp(key) && !(key in newProps)) {
						hostPatchProp(el, key, oldProps[key], null, namespace, parentComponent);
					}
				}
			}
			for (const key in newProps) {
				if (isReservedProp(key)) continue;
				const next = newProps[key];
				const prev = oldProps[key];
				if (next !== prev && key !== 'value') {
					hostPatchProp(el, key, prev, next, namespace, parentComponent);
				}
			}
			if ('value' in newProps) {
				hostPatchProp(el, 'value', oldProps.value, newProps.value, namespace);
			}
		}
	};
	const processFragment = (n1, n2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized) => {
		const fragmentStartAnchor = n2.el = n1 ? n1.el : hostCreateText('');
		const fragmentEndAnchor = n2.anchor = n1 ? n1.anchor : hostCreateText('');
		let { patchFlag, dynamicChildren, slotScopeIds: fragmentSlotScopeIds } = n2;
		if (fragmentSlotScopeIds) {
			slotScopeIds = slotScopeIds ? slotScopeIds.concat(fragmentSlotScopeIds) : fragmentSlotScopeIds;
		}
		if (n1 == null) {
			hostInsert(fragmentStartAnchor, container, anchor);
			hostInsert(fragmentEndAnchor, container, anchor);
			mountChildren(n2.children || [], container, fragmentEndAnchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
		} else {
			if (patchFlag > 0 && patchFlag & 64 && dynamicChildren && n1.dynamicChildren) {
				patchBlockChildren(n1.dynamicChildren, dynamicChildren, container, parentComponent, parentSuspense, namespace, slotScopeIds);
				if (n2.key != null || parentComponent && n2 === parentComponent.subTree) {
					traverseStaticChildren(n1, n2);
				}
			} else {
				patchChildren(n1, n2, container, fragmentEndAnchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
			}
		}
	};
	const processComponent = (n1, n2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized) => {
		n2.slotScopeIds = slotScopeIds;
		if (n1 == null) {
			if (n2.shapeFlag & 512) {
				parentComponent.ctx.activate(n2, container, anchor, namespace, optimized);
			} else {
				mountComponent(n2, container, anchor, parentComponent, parentSuspense, namespace, optimized);
			}
		} else {
			updateComponent(n1, n2, optimized);
		}
	};
	const mountComponent = (initialVNode, container, anchor, parentComponent, parentSuspense, namespace, optimized) => {
		const instance = initialVNode.component = createComponentInstance(initialVNode, parentComponent, parentSuspense);
		if (isKeepAlive(initialVNode)) {
			instance.ctx.renderer = internals;
		}
		{
			setupComponent(instance, 0, optimized);
		}
		if (instance.asyncDep) {
			parentSuspense && parentSuspense.registerDep(instance, setupRenderEffect, optimized);
			if (!initialVNode.el) {
				const placeholder = instance.subTree = createVNode(Comment);
				processCommentNode(0, placeholder, container, anchor);
			}
		} else {
			setupRenderEffect(instance, initialVNode, container, anchor, parentSuspense, namespace, optimized);
		}
	};
	const updateComponent = (n1, n2, optimized) => {
		const instance = n2.component = n1.component;
		if (shouldUpdateComponent(n1, n2, optimized)) {
			if (instance.asyncDep && !instance.asyncResolved) {
				updateComponentPreRender(instance, n2, optimized);
				return;
			} else {
				instance.next = n2;
				instance.update();
			}
		} else {
			n2.el = n1.el;
			instance.vnode = n2;
		}
	};
	const setupRenderEffect = (instance, initialVNode, container, anchor, parentSuspense, namespace, optimized) => {
		const componentUpdateFn = () => {
			if (!instance.isMounted) {
				let vnodeHook;
				const { el, props } = initialVNode;
				const { bm, m, parent, root, type } = instance;
				const isAsyncWrapperVNode = isAsyncWrapper(initialVNode);
				toggleRecurse(instance, false);
				if (bm) {
					invokeArrayFns(bm);
				}
				if (!isAsyncWrapperVNode && (vnodeHook = props && props.onVnodeBeforeMount)) {
					invokeVNodeHook(vnodeHook, parent, initialVNode);
				}
				toggleRecurse(instance, true);
				if (el && undefined) {
					const hydrateSubTree = () => {
						instance.subTree = renderComponentRoot(instance);
						undefined(el, instance.subTree, instance, parentSuspense, null);
					};
					if (isAsyncWrapperVNode && type.__asyncHydrate) {
						type.__asyncHydrate(el, instance, hydrateSubTree);
					} else {
						hydrateSubTree();
					}
				} else {
					if (root.ce) {
						root.ce._injectChildStyle(type);
					}
					const subTree = instance.subTree = renderComponentRoot(instance);
					patch(null, subTree, container, anchor, instance, parentSuspense, namespace);
					initialVNode.el = subTree.el;
				}
				if (m) {
					queuePostRenderEffect(m, parentSuspense);
				}
				if (!isAsyncWrapperVNode && (vnodeHook = props && props.onVnodeMounted)) {
					const scopedInitialVNode = initialVNode;
					queuePostRenderEffect(() => (invokeVNodeHook(vnodeHook, parent, scopedInitialVNode), undefined), parentSuspense);
				}
				if (initialVNode.shapeFlag & 256 || parent && isAsyncWrapper(parent.vnode) && parent.vnode.shapeFlag & 256) {
					instance.a && queuePostRenderEffect(instance.a, parentSuspense);
				}
				instance.isMounted = true;
				initialVNode = (container = (anchor = null, null), null);
			} else {
				let { next, bu, u, parent, vnode } = instance;
				{
					const nonHydratedAsyncRoot = locateNonHydratedAsyncRoot(instance);
					if (nonHydratedAsyncRoot) {
						if (next) {
							next.el = vnode.el;
							updateComponentPreRender(instance, next, optimized);
						}
						nonHydratedAsyncRoot.asyncDep.then(() => {
							if (!instance.isUnmounted) {
								componentUpdateFn();
							}
						});
						return;
					}
				}
				let originNext = next;
				let vnodeHook;
				toggleRecurse(instance, false);
				if (next) {
					next.el = vnode.el;
					updateComponentPreRender(instance, next, optimized);
				} else {
					next = vnode;
				}
				if (bu) {
					invokeArrayFns(bu);
				}
				if (vnodeHook = next.props && next.props.onVnodeBeforeUpdate) {
					invokeVNodeHook(vnodeHook, parent, next, vnode);
				}
				toggleRecurse(instance, true);
				const nextTree = renderComponentRoot(instance);
				const prevTree = instance.subTree;
				instance.subTree = nextTree;
				patch(prevTree, nextTree, hostParentNode(prevTree.el), getNextHostNode(prevTree), instance, parentSuspense, namespace);
				next.el = nextTree.el;
				if (originNext === null) {
					updateHOCHostEl(instance, nextTree.el);
				}
				if (u) {
					queuePostRenderEffect(u, parentSuspense);
				}
				if (vnodeHook = next.props && next.props.onVnodeUpdated) {
					queuePostRenderEffect(() => (invokeVNodeHook(vnodeHook, parent, next, vnode), undefined), parentSuspense);
				}
			}
		};
		instance.scope.on();
		const effect2 = instance.effect = new ReactiveEffect(componentUpdateFn);
		instance.scope.off();
		const update = instance.update = effect2.run.bind(effect2);
		const job = instance.job = effect2.runIfDirty.bind(effect2);
		job.i = instance;
		job.id = instance.uid;
		effect2.scheduler = () => (queueJob(job), undefined);
		toggleRecurse(instance, true);
		update();
	};
	const updateComponentPreRender = (instance, nextVNode, optimized) => {
		nextVNode.component = instance;
		const prevProps = instance.vnode.props;
		instance.vnode = nextVNode;
		instance.next = null;
		updateProps(instance, nextVNode.props, prevProps, optimized);
		updateSlots(instance, nextVNode.children, optimized);
		pauseTracking();
		flushPreFlushCbs(instance);
		resetTracking();
	};
	const patchChildren = (n1, n2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized = false) => {
		const c1 = n1 && n1.children;
		const prevShapeFlag = n1 ? n1.shapeFlag : 0;
		const c2 = n2.children;
		const { patchFlag, shapeFlag } = n2;
		if (patchFlag > 0) {
			if (patchFlag & 128) {
				patchKeyedChildren(c1, c2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
				return;
			} else if (patchFlag & 256) {
				patchUnkeyedChildren(c1, c2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
				return;
			}
		}
		if (shapeFlag & 8) {
			if (prevShapeFlag & 16) {
				unmountChildren(c1, parentComponent, parentSuspense);
			}
			if (c2 !== c1) {
				hostSetElementText(container, c2);
			}
		} else {
			if (prevShapeFlag & 16) {
				if (shapeFlag & 16) {
					patchKeyedChildren(c1, c2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
				} else {
					unmountChildren(c1, parentComponent, parentSuspense, true);
				}
			} else {
				if (prevShapeFlag & 8) {
					hostSetElementText(container, '');
				}
				if (shapeFlag & 16) {
					mountChildren(c2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
				}
			}
		}
	};
	const patchUnkeyedChildren = (c1, c2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized) => {
		c1 = c1 || EMPTY_ARR;
		c2 = c2 || EMPTY_ARR;
		const oldLength = c1.length;
		const newLength = c2.length;
		const commonLength = Math.min(oldLength, newLength);
		let i;
		for (i = 0; i < commonLength; i++) {
			const nextChild = c2[i] = optimized ? cloneIfMounted(c2[i]) : normalizeVNode(c2[i]);
			patch(c1[i], nextChild, container, null, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
		}
		if (oldLength > newLength) {
			unmountChildren(c1, parentComponent, parentSuspense, true, false, commonLength);
		} else {
			mountChildren(c2, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized, commonLength);
		}
	};
	const patchKeyedChildren = (c1, c2, container, parentAnchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized) => {
		let i = 0;
		const l2 = c2.length;
		let e1 = c1.length - 1;
		let e2 = l2 - 1;
		while (i <= e1 && i <= e2) {
			const n1 = c1[i];
			const n2 = c2[i] = optimized ? cloneIfMounted(c2[i]) : normalizeVNode(c2[i]);
			if (isSameVNodeType(n1, n2)) {
				patch(n1, n2, container, null, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
			} else {
				break;
			}
			i++;
		}
		while (i <= e1 && i <= e2) {
			const n1 = c1[e1];
			const n2 = c2[e2] = optimized ? cloneIfMounted(c2[e2]) : normalizeVNode(c2[e2]);
			if (isSameVNodeType(n1, n2)) {
				patch(n1, n2, container, null, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
			} else {
				break;
			}
			e1--;
			e2--;
		}
		if (i > e1) {
			if (i <= e2) {
				const nextPos = e2 + 1;
				const anchor = nextPos < l2 ? c2[nextPos].el : parentAnchor;
				while (i <= e2) {
					patch(null, c2[i] = optimized ? cloneIfMounted(c2[i]) : normalizeVNode(c2[i]), container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
					i++;
				}
			}
		} else if (i > e2) {
			while (i <= e1) {
				unmount(c1[i], parentComponent, parentSuspense, true);
				i++;
			}
		} else {
			const s1 = i;
			const s2 = i;
			const keyToNewIndexMap = new Map();
			for (i = s2; i <= e2; i++) {
				const nextChild = c2[i] = optimized ? cloneIfMounted(c2[i]) : normalizeVNode(c2[i]);
				if (nextChild.key != null) {
					keyToNewIndexMap.set(nextChild.key, i);
				}
			}
			let j;
			let patched = 0;
			const toBePatched = e2 - s2 + 1;
			let moved = false;
			let maxNewIndexSoFar = 0;
			const newIndexToOldIndexMap = new Array(toBePatched);
			for (i = 0; i < toBePatched; i++) newIndexToOldIndexMap[i] = 0;
			for (i = s1; i <= e1; i++) {
				const prevChild = c1[i];
				if (patched >= toBePatched) {
					unmount(prevChild, parentComponent, parentSuspense, true);
					continue;
				}
				let newIndex;
				if (prevChild.key != null) {
					newIndex = keyToNewIndexMap.get(prevChild.key);
				} else {
					for (j = s2; j <= e2; j++) {
						if (newIndexToOldIndexMap[j - s2] === 0 && (isSameVNodeType(prevChild, c2[j]), true)) {
							newIndex = j;
							break;
						}
					}
				}
				if (newIndex === undefined) {
					unmount(prevChild, parentComponent, parentSuspense, true);
				} else {
					newIndexToOldIndexMap[newIndex - s2] = i + 1;
					if (newIndex >= maxNewIndexSoFar) {
						maxNewIndexSoFar = newIndex;
					} else {
						moved = true;
					}
					patch(prevChild, c2[newIndex], container, null, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
					patched++;
				}
			}
			const increasingNewIndexSequence = moved ? getSequence(newIndexToOldIndexMap) : EMPTY_ARR;
			j = increasingNewIndexSequence.length - 1;
			for (i = toBePatched - 1; i >= 0; i--) {
				const nextIndex = s2 + i;
				const nextChild = c2[nextIndex];
				const anchor = nextIndex + 1 < l2 ? c2[nextIndex + 1].el : parentAnchor;
				if (newIndexToOldIndexMap[i] === 0) {
					patch(null, nextChild, container, anchor, parentComponent, parentSuspense, namespace, slotScopeIds, optimized);
				} else if (moved) {
					if (j < 0 || i !== increasingNewIndexSequence[j]) {
						move(nextChild, container, anchor, 2);
					} else {
						j--;
					}
				}
			}
		}
	};
	const move = (vnode, container, anchor, moveType, parentSuspense = null) => {
		const { el, type, transition, children, shapeFlag } = vnode;
		if (shapeFlag & 6) {
			move(vnode.component.subTree, container, anchor, moveType);
			return;
		}
		if (shapeFlag & 128) {
			vnode.suspense.move(container, anchor, moveType);
			return;
		}
		if (shapeFlag & 64) {
			type.move(vnode, container, anchor, internals);
			return;
		}
		if (type === Fragment) {
			hostInsert(el, container, anchor);
			for (let i = 0; i < children.length; i++) {
				move(children[i], container, anchor, moveType);
			}
			hostInsert(vnode.anchor, container, anchor);
			return;
		}
		if (type === Static) {
			moveStaticNode(vnode, container, anchor);
			return;
		}
		const needTransition2 = moveType !== 2 && shapeFlag & 1 && transition;
		if (needTransition2) {
			if (moveType === 0) {
				transition.beforeEnter(el);
				hostInsert(el, container, anchor);
				queuePostRenderEffect(() => transition.enter(el), parentSuspense);
			} else {
				const { leave, delayLeave, afterLeave } = transition;
				const remove22 = () => hostInsert(el, container, anchor);
				const performLeave = () => {
					leave(el, () => {
						remove22();
						afterLeave && afterLeave();
					});
				};
				if (delayLeave) {
					delayLeave(el, remove22, performLeave);
				} else {
					performLeave();
				}
			}
		} else {
			hostInsert(el, container, anchor);
		}
	};
	const unmount = (vnode, parentComponent, parentSuspense, doRemove = false, optimized = false) => {
		const { type, props, ref: ref3, children, dynamicChildren, shapeFlag, patchFlag, dirs, cacheIndex } = vnode;
		if (patchFlag === -2) {
			optimized = false;
		}
		if (ref3 != null) {
			setRef(ref3, null, parentSuspense, vnode, true);
		}
		if (cacheIndex != null) {
			parentComponent.renderCache[cacheIndex] = undefined;
		}
		if (shapeFlag & 256) {
			parentComponent.ctx.deactivate(vnode);
			return;
		}
		const shouldInvokeDirs = shapeFlag & 1 && dirs;
		const shouldInvokeVnodeHook = !isAsyncWrapper(vnode);
		let vnodeHook;
		if (shouldInvokeVnodeHook && (vnodeHook = props && props.onVnodeBeforeUnmount)) {
			invokeVNodeHook(vnodeHook, parentComponent, vnode);
		}
		if (shapeFlag & 6) {
			unmountComponent(vnode.component, parentSuspense, doRemove);
		} else {
			if (shapeFlag & 128) {
				vnode.suspense.unmount(parentSuspense, doRemove);
				return;
			}
			if (shouldInvokeDirs) {
				invokeDirectiveHook(vnode, null, parentComponent, 'beforeUnmount');
			}
			if (shapeFlag & 64) {
				vnode.type.remove(vnode, parentComponent, parentSuspense, internals, doRemove);
			} else if (dynamicChildren && !dynamicChildren.hasOnce && (type !== Fragment || patchFlag > 0 && patchFlag & 64)) {
				unmountChildren(dynamicChildren, parentComponent, parentSuspense, false, true);
			} else if (type === Fragment && patchFlag & (128 | 256) || !optimized && shapeFlag & 16) {
				unmountChildren(children, parentComponent, parentSuspense);
			}
			if (doRemove) {
				remove2(vnode);
			}
		}
		if (shouldInvokeVnodeHook && (vnodeHook = props && props.onVnodeUnmounted) || shouldInvokeDirs) {
			queuePostRenderEffect(() => {
				vnodeHook && invokeVNodeHook(vnodeHook, parentComponent, vnode);
				shouldInvokeDirs && invokeDirectiveHook(vnode, null, parentComponent, 'unmounted');
			}, parentSuspense);
		}
	};
	const remove2 = (vnode) => {
		const { type, el, anchor, transition } = vnode;
		if (type === Fragment) {
			{
				removeFragment(el, anchor);
			}
			return;
		}
		if (type === Static) {
			removeStaticNode(vnode);
			return;
		}
		const performRemove = () => {
			hostRemove(el);
			if (transition && !transition.persisted && transition.afterLeave) {
				transition.afterLeave();
			}
		};
		if (vnode.shapeFlag & 1 && transition && !transition.persisted) {
			const { leave, delayLeave } = transition;
			const performLeave = () => leave(el, performRemove);
			if (delayLeave) {
				delayLeave(vnode.el, performRemove, performLeave);
			} else {
				performLeave();
			}
		} else {
			performRemove();
		}
	};
	const removeFragment = (cur, end) => {
		let next;
		while (cur !== end) {
			next = hostNextSibling(cur);
			hostRemove(cur);
			cur = next;
		}
		hostRemove(end);
	};
	const unmountComponent = (instance, parentSuspense, doRemove) => {
		const { bum, scope, job, subTree, um, m, a } = instance;
		invalidateMount(m);
		invalidateMount(a);
		if (bum) {
			invokeArrayFns(bum);
		}
		scope.stop();
		if (job) {
			job.flags |= 8;
			unmount(subTree, instance, parentSuspense, doRemove);
		}
		if (um) {
			queuePostRenderEffect(um, parentSuspense);
		}
		queuePostRenderEffect(() => {
			instance.isUnmounted = true;
		}, parentSuspense);
		if (parentSuspense && parentSuspense.pendingBranch && !parentSuspense.isUnmounted && instance.asyncDep && !instance.asyncResolved && (instance.suspenseId, parentSuspense.pendingId, true)) {
			parentSuspense.deps--;
			if (parentSuspense.deps === 0) {
				parentSuspense.resolve();
			}
		}
	};
	const unmountChildren = (children, parentComponent, parentSuspense, doRemove = false, optimized = false, start = 0) => {
		for (let i = start; i < children.length; i++) {
			unmount(children[i], parentComponent, parentSuspense, doRemove, optimized);
		}
	};
	const getNextHostNode = (vnode) => {
		if (vnode.shapeFlag & 6) {
			return getNextHostNode(vnode.component.subTree);
		}
		if (vnode.shapeFlag & 128) {
			return vnode.suspense.next();
		}
		const el = hostNextSibling(vnode.anchor || vnode.el);
		const teleportEnd = el && el[TeleportEndKey];
		return teleportEnd ? hostNextSibling(teleportEnd) : el;
	};
	let isFlushing = false;
	const render = (vnode, container, namespace) => {
		if (vnode == null) {
			if (container._vnode) {
				unmount(container._vnode, null, null, true);
			}
		} else {
			patch(container._vnode || null, vnode, container, null, null, null, namespace);
		}
		container._vnode = vnode;
		if (!isFlushing) {
			isFlushing = true;
			flushPreFlushCbs();
			flushPostFlushCbs();
			isFlushing = false;
		}
	};
	const internals = {
		p: patch,
		um: unmount,
		m: move,
		r: remove2,
		mt: mountComponent,
		mc: mountChildren,
		pc: patchChildren,
		pbc: patchBlockChildren,
		n: getNextHostNode,
		o: options
	};
	return { createApp: createAppAPI(render) };
}
function resolveChildrenNamespace({ type, props }, currentNamespace) {
	return currentNamespace === 'svg' && type === 'foreignObject' || currentNamespace === 'mathml' && type === 'annotation-xml' && props && props.encoding && props.encoding.includes('html') ? undefined : currentNamespace;
}
function toggleRecurse({ effect: effect2, job }, allowed) {
	if (allowed) {
		effect2.flags |= 32;
		job.flags |= 4;
	} else {
		effect2.flags &= ~32;
		job.flags &= ~4;
	}
}
function needTransition(parentSuspense, transition) {
	return (!parentSuspense || parentSuspense && !parentSuspense.pendingBranch) && transition && !transition.persisted;
}
function traverseStaticChildren(n1, n2) {
	const ch1 = n1.children;
	const ch2 = n2.children;
	if (isArray(ch1) && isArray(ch2)) {
		for (let i = 0; i < ch1.length; i++) {
			const c1 = ch1[i];
			let c2 = ch2[i];
			if (c2.shapeFlag & 1 && !c2.dynamicChildren) {
				if (c2.patchFlag <= 0 || c2.patchFlag === 32) {
					c2 = ch2[i] = cloneIfMounted(ch2[i]);
					c2.el = c1.el;
				}
			}
			if (c2.type === Text) {
				c2.el = c1.el;
			}
		}
	}
}
function getSequence(arr) {
	const p2 = arr.slice();
	const result = [0];
	let i, j, u, v, c;
	const len = arr.length;
	for (i = 0; i < len; i++) {
		const arrI = arr[i];
		if (arrI !== 0) {
			j = result[result.length - 1];
			if (arr[j] < arrI) {
				p2[i] = j;
				result.push(i);
				continue;
			}
			u = 0;
			v = result.length - 1;
			while (u < v) {
				c = u + v >> 1;
				if (arr[result[c]] < arrI) {
					u = c + 1;
				} else {
					v = c;
				}
			}
			if (arrI < arr[result[u]]) {
				if (u > 0) {
					p2[i] = result[u - 1];
				}
				result[u] = i;
			}
		}
	}
	u = result.length;
	v = result[u - 1];
	while (u-- > 0) {
		result[u] = v;
		v = p2[v];
	}
	return result;
}
function locateNonHydratedAsyncRoot(instance) {
	const subComponent = instance.subTree.component;
	if (subComponent) {
		if (subComponent.asyncDep && !subComponent.asyncResolved) {
			return subComponent;
		} else {
			return locateNonHydratedAsyncRoot(subComponent);
		}
	}
}
function invalidateMount(hooks) {
	if (hooks) {
		for (let i = 0; i < hooks.length; i++) hooks[i].flags |= 8;
	}
}
const ssrContextKey = Symbol.for('v-scx');
const useSSRContext = () => {
	{
		const ctx = inject(ssrContextKey);
		return ctx;
	}
};
function watch(source, cb, options) {
	return doWatch(source, cb, options);
}
function doWatch(source, cb, options = EMPTY_OBJ) {
	const { immediate, deep, flush, once } = options;
	const baseWatchOptions = extend({}, options);
	const runsImmediately = cb && immediate || !cb && flush !== 'post';
	let ssrCleanup;
	if (isInSSRComponentSetup) {
		if (flush === 'sync') {
			const ctx = useSSRContext();
			ssrCleanup = ctx.__watcherHandles || (ctx.__watcherHandles = []);
		} else if (!runsImmediately) {
			const watchStopHandle = () => {};
			watchStopHandle.stop = NOOP;
			watchStopHandle.resume = NOOP;
			watchStopHandle.pause = NOOP;
			return watchStopHandle;
		}
	}
	const instance = currentInstance;
	baseWatchOptions.call = (fn, type, args) => callWithAsyncErrorHandling(fn, instance, type, args);
	let isPre = false;
	if (flush === 'post') {
		baseWatchOptions.scheduler = (job) => {
			queuePostRenderEffect(job, instance && instance.suspense);
		};
	} else if (flush !== 'sync') {
		isPre = true;
		baseWatchOptions.scheduler = (job, isFirstRun) => {
			if (isFirstRun) {
				job();
			} else {
				queueJob(job);
			}
		};
	}
	baseWatchOptions.augmentJob = (job) => {
		if (cb) {
			job.flags |= 4;
		}
		if (isPre) {
			job.flags |= 2;
			if (instance) {
				job.id = instance.uid;
				job.i = instance;
			}
		}
	};
	const watchHandle = watch$1(source, cb, baseWatchOptions);
	if (isInSSRComponentSetup) {
		if (ssrCleanup) {
			ssrCleanup.push(watchHandle);
		} else if (runsImmediately) {
			watchHandle();
		}
	}
	return watchHandle;
}
function instanceWatch(source, value, options) {
	const publicThis = this.proxy;
	const getter = isString(source) ? source.includes('.') ? createPathGetter(publicThis, source) : () => publicThis[source] : source.bind(publicThis, publicThis);
	let cb;
	if (isFunction(value)) {
		cb = value;
	} else {
		cb = value.handler;
		options = value;
	}
	const reset = setCurrentInstance(this);
	const res = doWatch(getter, cb.bind(publicThis), options);
	reset();
	return res;
}
function createPathGetter(ctx, path) {
	const segments = path.split('.');
	return () => {
		let cur = ctx;
		for (let i = 0; i < segments.length && cur; i++) {
			cur = cur[segments[i]];
		}
		return cur;
	};
}
const getModelModifiers = (props, modelName) => {
	return modelName === 'modelValue' || modelName === 'model-value' ? props.modelModifiers : props[`${modelName}Modifiers`] || props[`${camelize(modelName)}Modifiers`] || props[`${hyphenate(modelName)}Modifiers`];
};
function emit(instance, event, ...rawArgs) {
	if (instance.isUnmounted) return;
	const props = instance.vnode.props || EMPTY_OBJ;
	let args = rawArgs;
	const isModelListener2 = event.startsWith('update:');
	const modifiers = isModelListener2 && getModelModifiers(props, event.slice(7));
	if (modifiers) {
		if (modifiers.trim) {
			args = rawArgs.map((a) => isString(a) ? a.trim() : a);
		}
		if (modifiers.number) {
			args = rawArgs.map(looseToNumber);
		}
	}
	let handlerName;
	let handler = props[handlerName = toHandlerKey(event)] || props[handlerName = toHandlerKey(camelize(event))];
	if (!handler && isModelListener2) {
		handler = props[handlerName = toHandlerKey(hyphenate(event))];
	}
	if (handler) {
		callWithAsyncErrorHandling(handler, instance, 6, args);
	}
	const onceHandler = props[handlerName + 'Once'];
	if (onceHandler) {
		if (!instance.emitted) {
			instance.emitted = {};
		} else if (instance.emitted[handlerName]) {
			return;
		}
		instance.emitted[handlerName] = true;
		callWithAsyncErrorHandling(onceHandler, instance, 6, args);
	}
}
function normalizeEmitsOptions(comp, appContext, asMixin = false) {
	const cache = appContext.emitsCache;
	const cached = cache.get(comp);
	if (cached !== undefined) {
		return cached;
	}
	const raw = comp.emits;
	let normalized = {};
	let hasExtends = false;
	if (!isFunction(comp)) {
		const extendEmits = (raw2) => {
			const normalizedFromExtend = normalizeEmitsOptions(raw2, appContext, true);
			if (normalizedFromExtend) {
				hasExtends = true;
				extend(normalized, normalizedFromExtend);
			}
		};
		if (!asMixin && appContext.mixins.length) {
			appContext.mixins.forEach(extendEmits);
		}
		if (comp.extends) {
			extendEmits(comp.extends);
		}
		if (comp.mixins) {
			comp.mixins.forEach(extendEmits);
		}
	}
	if (!raw && !hasExtends) {
		if (isObject(comp)) {
			cache.set(comp, null);
		}
		return null;
	}
	if (isArray(raw)) {
		raw.forEach((key) => (normalized[key] = null, null));
	} else {
		extend(normalized, raw);
	}
	if (isObject(comp)) {
		cache.set(comp, normalized);
	}
	return normalized;
}
function isEmitListener(options, key) {
	if (!options || !isOn(key)) {
		return false;
	}
	key = key.slice(2).replace(/Once$/, '');
	return hasOwn(options, key[0].toLowerCase() + key.slice(1)) || hasOwn(options, hyphenate(key)) || hasOwn(options, key);
}
function renderComponentRoot(instance) {
	const { type: Component, vnode, proxy, withProxy, propsOptions: [propsOptions], slots, attrs, emit: emit2, render, renderCache, props, data, setupState, ctx, inheritAttrs } = instance;
	const prev = setCurrentRenderingInstance(instance);
	let result;
	let fallthroughAttrs;
	try {
		if (vnode.shapeFlag & 4) {
			const proxyToUse = withProxy || proxy;
			const thisProxy = proxyToUse;
			result = normalizeVNode(render.call(thisProxy, proxyToUse, renderCache, props, setupState, data, ctx));
			fallthroughAttrs = attrs;
		} else {
			const render2 = Component;
			result = normalizeVNode(render2.length > 1 ? render2(props, {
				attrs,
				slots,
				emit: emit2
			}) : render2(props, null));
			fallthroughAttrs = Component.props ? attrs : getFunctionalFallthrough(attrs);
		}
	} catch (err) {
		blockStack.length = 0;
		handleError(err, instance, 1);
		result = createVNode(Comment);
	}
	let root = result;
	if (fallthroughAttrs && inheritAttrs !== false) {
		const keys = Object.keys(fallthroughAttrs);
		const { shapeFlag } = root;
		if (keys.length) {
			if (shapeFlag & (1 | 6)) {
				if (propsOptions && keys.some(isModelListener)) {
					fallthroughAttrs = filterModelListeners(fallthroughAttrs, propsOptions);
				}
				root = cloneVNode(root, fallthroughAttrs, false, true);
			}
		}
	}
	if (vnode.dirs) {
		root = cloneVNode(root, null, 0, true);
		root.dirs = root.dirs ? root.dirs.concat(vnode.dirs) : vnode.dirs;
	}
	if (vnode.transition) {
		setTransitionHooks(root, vnode.transition);
	}
	{
		result = root;
	}
	setCurrentRenderingInstance(prev);
	return result;
}
const getFunctionalFallthrough = (attrs) => {
	let res;
	for (const key in attrs) {
		if (key === 'class' || key === 'style' || isOn(key)) {
			(res || (res = {}))[key] = attrs[key];
		}
	}
	return res;
};
const filterModelListeners = (attrs, props) => {
	const res = {};
	for (const key in attrs) {
		if (!isModelListener(key) || !(key.slice(9) in props)) {
			res[key] = attrs[key];
		}
	}
	return res;
};
function shouldUpdateComponent(prevVNode, nextVNode, optimized) {
	const { props: prevProps, children: prevChildren, component } = prevVNode;
	const { props: nextProps, children: nextChildren, patchFlag } = nextVNode;
	const __unused_4B46 = component.emitsOptions;
	if (nextVNode.dirs || nextVNode.transition) {
		return true;
	}
	if (optimized && patchFlag >= 0) {
		if (patchFlag & 1024) {
			return true;
		}
		if (patchFlag & 16) {
			if (!prevProps) {
				return !!nextProps;
			}
			return hasPropsChanged(prevProps, nextProps), false;
		} else if (patchFlag & 8) {
			const dynamicProps = nextVNode.dynamicProps;
			for (let i = 0; i < dynamicProps.length; i++) {
				const key = dynamicProps[i];
				{
					nextProps[key], prevProps[key];
				}
			}
		}
	} else {
		if (prevChildren || nextChildren) {
			if (!nextChildren || !nextChildren.$stable) {
				return true;
			}
		}
		if (prevProps === nextProps) {
			return false;
		}
		if (!prevProps) {
			return !!nextProps;
		}
		if (!nextProps) {
			return true;
		}
		return hasPropsChanged(prevProps, nextProps), false;
	}
	return false;
}
function hasPropsChanged(prevProps, nextProps) {
	const nextKeys = Object.keys(nextProps);
	{
		nextKeys.length, Object.keys(prevProps).length;
	}
	for (let i = 0; i < nextKeys.length; i++) {
		const key = nextKeys[i];
		{
			nextProps[key], prevProps[key];
		}
	}
	return;
}
function updateHOCHostEl({ vnode, parent }, el) {
	while (parent) {
		const root = parent.subTree;
		if (root.suspense && root.suspense.activeBranch === vnode) {
			root.el = vnode.el;
		}
		if (root === vnode) {
			(vnode = parent.vnode).el = el;
			parent = parent.parent;
		} else {
			break;
		}
	}
}
const isSuspense = (type) => type.__isSuspense;
function queueEffectWithSuspense(fn, suspense) {
	if (suspense && suspense.pendingBranch) {
		if (isArray(fn)) {
			suspense.effects.push(...fn);
		} else {
			suspense.effects.push(fn);
		}
	} else {
		queuePostFlushCb(fn);
	}
}
const Fragment = Symbol.for('v-fgt');
const Text = Symbol.for('v-txt');
const Comment = Symbol.for('v-cmt');
const Static = Symbol.for('v-stc');
const blockStack = [];
let currentBlock = null;
function openBlock() {
	blockStack.push(currentBlock = []);
}
function closeBlock() {
	blockStack.pop();
	currentBlock = blockStack[blockStack.length - 1] || null;
}
let isBlockTreeEnabled = 1;
function setBlockTracking(value) {
	isBlockTreeEnabled += value;
	if (value < 0 && currentBlock) {
		currentBlock.hasOnce = true;
	}
}
function setupBlock(vnode) {
	vnode.dynamicChildren = isBlockTreeEnabled > 0 ? currentBlock || EMPTY_ARR : null;
	closeBlock();
	if (isBlockTreeEnabled > 0 && currentBlock) {
		currentBlock.push(vnode);
	}
	return vnode;
}
function createElementBlock(__unused_CADF, __unused_EA66, children) {
	return setupBlock(createBaseVNode('div', null, children, undefined, undefined, undefined, true));
}
function isVNode(value) {
	return value ? value.__v_isVNode === true : false;
}
function isSameVNodeType(n1, n2) {
	return n1.type === n2.type && n1.key === n2.key;
}
const normalizeKey = ({ key }) => key != null ? key : null;
const normalizeRef = ({ ref: ref3, ref_key, ref_for }) => {
	if (typeof ref3 === 'number') {
		ref3 = '' + ref3;
	}
	return ref3 != null ? isString(ref3) || isRef(ref3) || isFunction(ref3) ? {
		i: currentRenderingInstance,
		r: ref3,
		k: ref_key,
		f: !!ref_for
	} : ref3 : null;
};
function createBaseVNode(type, props = null, children = null, patchFlag = 0, dynamicProps = null, shapeFlag = 'div' === Fragment ? 0 : 1, isBlockNode = false, needFullChildrenNormalization = false) {
	const vnode = {
		__v_isVNode: true,
		__v_skip: true,
		type,
		props,
		key: props && normalizeKey(props),
		ref: props && normalizeRef(props),
		scopeId: currentScopeId,
		slotScopeIds: null,
		children,
		component: null,
		suspense: null,
		ssContent: null,
		ssFallback: null,
		dirs: null,
		transition: null,
		el: null,
		anchor: null,
		target: null,
		targetStart: null,
		targetAnchor: null,
		staticCount: 0,
		shapeFlag,
		patchFlag,
		dynamicProps,
		dynamicChildren: null,
		appContext: null,
		ctx: currentRenderingInstance
	};
	if (needFullChildrenNormalization) {
		normalizeChildren(vnode, children);
		if (shapeFlag & 128) {
			type.normalize(vnode);
		}
	} else {
		{
			vnode.shapeFlag |= 8;
		}
	}
	if (isBlockTreeEnabled > 0 && !isBlockNode && currentBlock && (vnode.patchFlag > 0 || shapeFlag & 6) && vnode.patchFlag !== 32) {
		currentBlock.push(vnode);
	}
	return vnode;
}
const createVNode = _createVNode;
function _createVNode(type, props = null, children = null, patchFlag = 0, dynamicProps = null, isBlockNode = false) {
	if (!type || type === NULL_DYNAMIC_COMPONENT) {
		type = Comment;
	}
	if (isVNode(type)) {
		const cloned = cloneVNode(type, props, true);
		if (children) {
			normalizeChildren(cloned, children);
		}
		if (isBlockTreeEnabled > 0 && !isBlockNode && currentBlock) {
			if (cloned.shapeFlag & 6) {
				currentBlock[currentBlock.indexOf(type)] = cloned;
			} else {
				currentBlock.push(cloned);
			}
		}
		cloned.patchFlag = -2;
		return cloned;
	}
	if (isClassComponent(type)) {
		type = type.__vccOpts;
	}
	if (props) {
		props = guardReactiveProps(props);
		let { class: klass, style } = props;
		if (klass && !isString(klass)) {
			props.class = normalizeClass(klass);
		}
		if (isObject(style)) {
			if (isProxy(style) && !isArray(style)) {
				style = extend({}, style);
			}
			props.style = normalizeStyle(style);
		}
	}
	const shapeFlag = isString(type) ? 1 : isSuspense(type) ? 128 : isTeleport(type) ? 64 : isObject(type) ? 4 : isFunction(type) ? 2 : 0;
	return createBaseVNode(type, props, children, patchFlag, dynamicProps, shapeFlag, isBlockNode, true);
}
function guardReactiveProps(props) {
	if (!props) return null;
	return isProxy(props) || isInternalObject(props) ? extend({}, props) : props;
}
function cloneVNode(vnode, extraProps, mergeRef = false, cloneTransition = false) {
	const { props, ref: ref3, patchFlag, children, transition } = vnode;
	const mergedProps = extraProps ? mergeProps(props || {}, extraProps) : props;
	const cloned = {
		__v_isVNode: true,
		__v_skip: true,
		type: vnode.type,
		props: mergedProps,
		key: mergedProps && normalizeKey(mergedProps),
		ref: extraProps && extraProps.ref ? mergeRef && ref3 ? isArray(ref3) ? ref3.concat(normalizeRef(extraProps)) : [ref3, normalizeRef(extraProps)] : normalizeRef(extraProps) : ref3,
		scopeId: vnode.scopeId,
		slotScopeIds: vnode.slotScopeIds,
		children,
		target: vnode.target,
		targetStart: vnode.targetStart,
		targetAnchor: vnode.targetAnchor,
		staticCount: vnode.staticCount,
		shapeFlag: vnode.shapeFlag,
		patchFlag: extraProps && vnode.type !== Fragment ? patchFlag === -1 ? 16 : patchFlag | 16 : patchFlag,
		dynamicProps: vnode.dynamicProps,
		dynamicChildren: vnode.dynamicChildren,
		appContext: vnode.appContext,
		dirs: vnode.dirs,
		transition,
		component: vnode.component,
		suspense: vnode.suspense,
		ssContent: vnode.ssContent && cloneVNode(vnode.ssContent),
		ssFallback: vnode.ssFallback && cloneVNode(vnode.ssFallback),
		el: vnode.el,
		anchor: vnode.anchor,
		ctx: vnode.ctx,
		ce: vnode.ce
	};
	if (transition && cloneTransition) {
		setTransitionHooks(cloned, transition.clone(cloned));
	}
	return cloned;
}
function createTextVNode(text = ' ') {
	return createVNode(Text, null, text, 0);
}
function normalizeVNode(child) {
	if (child == null || typeof child === 'boolean') {
		return createVNode(Comment);
	} else if (isArray(child)) {
		return createVNode(Fragment, null, child.slice());
	} else if (isVNode(child)) {
		return cloneIfMounted(child);
	} else {
		return createVNode(Text, null, String(child));
	}
}
function cloneIfMounted(child) {
	return child.el === null && child.patchFlag !== -1 || child.memo ? child : cloneVNode(child);
}
function normalizeChildren(vnode, children) {
	let type = 0;
	const { shapeFlag } = vnode;
	if (children == null) {
		children = null;
	} else if (isArray(children)) {
		type = 16;
	} else if (typeof children === 'object') {
		if (shapeFlag & (1 | 64)) {
			const slot = children.default;
			if (slot) {
				slot._c && (slot._d = false);
				normalizeChildren(vnode, slot());
				slot._c && (slot._d = true);
			}
			return;
		} else {
			type = 32;
			const slotFlag = children._;
			if (!slotFlag && !isInternalObject(children)) {
				children._ctx = currentRenderingInstance;
			} else if (slotFlag === 3 && currentRenderingInstance) {
				if (currentRenderingInstance.slots._ === 1) {
					children._ = 1;
				} else {
					children._ = 2;
					vnode.patchFlag |= 1024;
				}
			}
		}
	} else if (isFunction(children)) {
		children = {
			default: children,
			_ctx: currentRenderingInstance
		};
		type = 32;
	} else {
		children = String(children);
		if (shapeFlag & 64) {
			type = 16;
			children = [createTextVNode(children)];
		} else {
			type = 8;
		}
	}
	vnode.children = children;
	vnode.shapeFlag |= type;
}
function mergeProps(...args) {
	const ret = {};
	for (let i = 0; i < 2; i++) {
		const toMerge = args[i];
		for (const key in toMerge) {
			if (key === 'class') {
				if (ret.class !== toMerge.class) {
					ret.class = normalizeClass([ret.class, toMerge.class]);
				}
			} else if (key === 'style') {
				ret.style = normalizeStyle([ret.style, toMerge.style]);
			} else if (isOn(key)) {
				const existing = ret[key];
				const incoming = toMerge[key];
				if (incoming && existing !== incoming && !(isArray(existing) && existing.includes(incoming))) {
					ret[key] = existing ? [].concat(existing, incoming) : incoming;
				}
			} else if (key !== '') {
				ret[key] = toMerge[key];
			}
		}
	}
	return ret;
}
function invokeVNodeHook(hook, instance, vnode, prevVNode = null) {
	callWithAsyncErrorHandling(hook, instance, 7, [vnode, prevVNode]);
}
const emptyAppContext = createAppContext();
let uid = 0;
function createComponentInstance(vnode, parent, suspense) {
	const type = vnode.type;
	const appContext = (parent ? parent.appContext : vnode.appContext) || emptyAppContext;
	const instance = {
		uid: uid++,
		vnode,
		type,
		parent,
		appContext,
		next: null,
		subTree: null,
		effect: null,
		update: null,
		job: null,
		scope: new EffectScope(true),
		render: null,
		proxy: null,
		exposed: null,
		exposeProxy: null,
		withProxy: null,
		provides: parent ? parent.provides : Object.create(appContext.provides),
		ids: parent ? parent.ids : [
			'',
			0,
			0
		],
		accessCache: null,
		renderCache: [],
		components: null,
		directives: null,
		propsOptions: normalizePropsOptions(type, appContext),
		emitsOptions: normalizeEmitsOptions(type, appContext),
		emit: null,
		emitted: null,
		propsDefaults: EMPTY_OBJ,
		inheritAttrs: type.inheritAttrs,
		data: EMPTY_OBJ,
		props: EMPTY_OBJ,
		attrs: EMPTY_OBJ,
		slots: EMPTY_OBJ,
		refs: EMPTY_OBJ,
		setupState: EMPTY_OBJ,
		setupContext: null,
		suspense,
		suspenseId: suspense ? suspense.pendingId : 0,
		asyncDep: null,
		asyncResolved: false,
		isMounted: false,
		isUnmounted: false,
		isDeactivated: false,
		bc: null,
		c: null,
		bm: null,
		m: null,
		bu: null,
		u: null,
		um: null,
		bum: null,
		da: null,
		a: null,
		rtg: null,
		rtc: null,
		ec: null,
		sp: null
	};
	{
		instance.ctx = { _: instance };
	}
	instance.root = parent ? parent.root : instance;
	instance.emit = emit.bind(null, instance);
	if (vnode.ce) {
		vnode.ce(instance);
	}
	return instance;
}
let currentInstance = null;
let internalSetCurrentInstance;
{
	const g = getGlobalThis();
	const registerGlobalSetter = (key, setter) => {
		let setters;
		if (!(setters = g[key])) setters = g[key] = [];
		setters.push(setter);
		return (v) => {
			if (setters.length > 1) setters.forEach((set) => set(v));
else setters[0](v);
		};
	};
	internalSetCurrentInstance = registerGlobalSetter('__VUE_INSTANCE_SETTERS__', (v) => currentInstance = v);
	registerGlobalSetter('__VUE_SSR_SETTERS__', (v) => isInSSRComponentSetup = v);
}
const setCurrentInstance = (instance) => {
	const prev = currentInstance;
	internalSetCurrentInstance(instance);
	instance.scope.on();
	return () => {
		instance.scope.off();
		internalSetCurrentInstance(prev);
	};
};
const unsetCurrentInstance = () => {
	currentInstance && currentInstance.scope.off();
	internalSetCurrentInstance(null);
};
function isStatefulComponent(instance) {
	return instance.vnode.shapeFlag & 4;
}
let isInSSRComponentSetup = false;
function setupComponent(instance, __unused_0D9F, optimized = false) {
	const { props, children } = instance.vnode;
	const isStateful = isStatefulComponent(instance);
	initProps(instance, props, isStateful);
	initSlots(instance, children, optimized);
	const __unused_3621 = isStateful && setupStatefulComponent(instance);
	return;
}
function setupStatefulComponent(instance) {
	const Component = instance.type;
	instance.accessCache = Object.create(null);
	instance.proxy = new Proxy(instance.ctx, PublicInstanceProxyHandlers);
	const { setup } = Component;
	if (setup) {
		pauseTracking();
		const setupContext = instance.setupContext = setup.length > 1 ? createSetupContext(instance) : null;
		const reset = setCurrentInstance(instance);
		const setupResult = callWithErrorHandling(setup, instance, 0, [instance.props, setupContext]);
		const isAsyncSetup = isPromise(setupResult);
		resetTracking();
		reset();
		if ((isAsyncSetup || instance.sp) && !isAsyncWrapper(instance)) {
			markAsyncBoundary(instance);
		}
		if (isAsyncSetup) {
			setupResult.then(unsetCurrentInstance, unsetCurrentInstance);
			{
				{
					instance.asyncDep = setupResult;
				}
			}
		} else {
			handleSetupResult(instance, setupResult);
		}
	} else {
		finishComponentSetup(instance);
	}
}
function handleSetupResult(instance, setupResult) {
	if (isFunction(setupResult)) {
		if (instance.type.__ssrInlineRender) {
			instance.ssrRender = setupResult;
		} else {
			instance.render = setupResult;
		}
	} else if (isObject(setupResult)) {
		instance.setupState = proxyRefs(setupResult);
	}
	finishComponentSetup(instance);
}
function finishComponentSetup(instance) {
	const Component = instance.type;
	if (!instance.render) {
		instance.render = Component.render || NOOP;
	}
	{
		const reset = setCurrentInstance(instance);
		pauseTracking();
		try {
			applyOptions(instance);
		} finally {
			resetTracking();
			reset();
		}
	}
}
const attrsProxyHandlers = { get(target, key) {
	track(target, 0, '');
	return target[key];
} };
function createSetupContext(instance) {
	const expose = (exposed) => {
		instance.exposed = exposed || {};
	};
	{
		return {
			attrs: new Proxy(instance.attrs, attrsProxyHandlers),
			slots: instance.slots,
			emit: instance.emit,
			expose
		};
	}
}
function getComponentPublicInstance(instance) {
	if (instance.exposed) {
		return instance.exposeProxy || (instance.exposeProxy = new Proxy(proxyRefs(markRaw(instance.exposed)), {
			get(target, key) {
				if (key in target) {
					return target[key];
				} else if (key in publicPropertiesMap) {
					return publicPropertiesMap[key](instance);
				}
			},
			has(target, key) {
				return key in target || key in publicPropertiesMap;
			}
		}));
	} else {
		return instance.proxy;
	}
}
function isClassComponent(value) {
	return isFunction(value) && '__vccOpts' in value;
}
const computed = (getterOrOptions) => {
	const c = computed$1(getterOrOptions, 0, isInSSRComponentSetup);
	return c;
};
let policy = undefined;
const tt = typeof window !== 'undefined' && window.trustedTypes;
if (tt) {
	try {
		policy = tt.createPolicy('vue', { createHTML: (val) => val });
	} catch {}
}
const unsafeToTrustedHTML = policy ? (val) => policy.createHTML(val) : (val) => val;
const svgNS = 'http://www.w3.org/2000/svg';
const mathmlNS = 'http://www.w3.org/1998/Math/MathML';
const doc = typeof document !== 'undefined' ? document : null;
const templateContainer = doc && doc.createElement('template');
const nodeOps = {
	insert: (child, parent, anchor) => {
		parent.insertBefore(child, anchor || null);
	},
	remove: (child) => {
		const parent = child.parentNode;
		if (parent) {
			parent.removeChild(child);
		}
	},
	createElement: (tag, namespace, is, props) => {
		const el = namespace === 'svg' ? doc.createElementNS(svgNS, tag) : namespace === 'mathml' ? doc.createElementNS(mathmlNS, tag) : is ? doc.createElement(tag, { is }) : doc.createElement(tag);
		if (tag === 'select' && props && props.multiple != null) {
			el.setAttribute('multiple', props.multiple);
		}
		return el;
	},
	createText: (text) => doc.createTextNode(text),
	createComment: (text) => doc.createComment(text),
	setText: (node, text) => {
		node.nodeValue = text;
	},
	setElementText: (el, text) => {
		el.textContent = text;
	},
	parentNode: (node) => node.parentNode,
	nextSibling: (node) => node.nextSibling,
	querySelector: (selector) => doc.querySelector(selector),
	setScopeId(el, id) {
		el.setAttribute(id, '');
	},
	insertStaticContent(content, parent, anchor, namespace, start, end) {
		const before = anchor ? anchor.previousSibling : parent.lastChild;
		if (start && (start === end || start.nextSibling)) {
			while (true) {
				parent.insertBefore(start.cloneNode(true), anchor);
				if (start === end || !(start = start.nextSibling)) break;
			}
		} else {
			templateContainer.innerHTML = unsafeToTrustedHTML(namespace === 'svg' ? `<svg>${content}</svg>` : namespace === 'mathml' ? `<math>${content}</math>` : content);
			const template = templateContainer.content;
			if (namespace === 'svg' || namespace === 'mathml') {
				const wrapper = template.firstChild;
				while (wrapper.firstChild) {
					template.appendChild(wrapper.firstChild);
				}
				template.removeChild(wrapper);
			}
			parent.insertBefore(template, anchor);
		}
		return [before ? before.nextSibling : parent.firstChild, anchor ? anchor.previousSibling : parent.lastChild];
	}
};
const vtcKey = Symbol('_vtc');
function patchClass(el, value, isSVG) {
	const transitionClasses = el[vtcKey];
	if (transitionClasses) {
		value = (value ? [value, ...transitionClasses] : [...transitionClasses]).join(' ');
	}
	if (value == null) {
		el.removeAttribute('class');
	} else if (isSVG) {
		el.setAttribute('class', value);
	} else {
		el.className = value;
	}
}
const vShowOriginalDisplay = Symbol('_vod');
const vShowHidden = Symbol('_vsh');
const CSS_VAR_TEXT = Symbol('');
const displayRE = /(^|;)\s*display\s*:/;
function patchStyle(el, prev, next) {
	const style = el.style;
	const isCssString = isString(next);
	let hasControlledDisplay = false;
	if (next && !isCssString) {
		if (prev) {
			if (!isString(prev)) {
				for (const key in prev) {
					if (next[key] == null) {
						setStyle(style, key, '');
					}
				}
			} else {
				for (const prevStyle of prev.split(';')) {
					const key = prevStyle.slice(0, prevStyle.indexOf(':')).trim();
					if (next[key] == null) {
						setStyle(style, key, '');
					}
				}
			}
		}
		for (const key in next) {
			if (key === 'display') {
				hasControlledDisplay = true;
			}
			setStyle(style, key, next[key]);
		}
	} else {
		if (isCssString) {
			if (prev !== next) {
				const cssVarText = style[CSS_VAR_TEXT];
				if (cssVarText) {
					next += ';' + cssVarText;
				}
				style.cssText = next;
				hasControlledDisplay = displayRE.test(next);
			}
		} else if (prev) {
			el.removeAttribute('style');
		}
	}
	if (vShowOriginalDisplay in el) {
		el[vShowOriginalDisplay] = hasControlledDisplay ? style.display : '';
		if (el[vShowHidden]) {
			style.display = 'none';
		}
	}
}
const importantRE = /\s*!important$/;
function setStyle(style, name, val) {
	if (isArray(val)) {
		val.forEach((v) => setStyle(style, name, v));
	} else {
		if (val == null) val = '';
		if (name.startsWith('--')) {
			style.setProperty(name, val);
		} else {
			const prefixed = autoPrefix(style, name);
			if (importantRE.test(val)) {
				style.setProperty(hyphenate(prefixed), val.replace(importantRE, ''), 'important');
			} else {
				style[prefixed] = val;
			}
		}
	}
}
const prefixes = [
	'Webkit',
	'Moz',
	'ms'
];
const prefixCache = {};
function autoPrefix(style, rawName) {
	const cached = prefixCache[rawName];
	if (cached) {
		return cached;
	}
	let name = camelize(rawName);
	if (name !== 'filter' && name in style) {
		return prefixCache[rawName] = name;
	}
	name = capitalize(name);
	for (let i = 0; i < 3; i++) {
		const prefixed = prefixes[i] + name;
		if (prefixed in style) {
			return prefixCache[rawName] = prefixed;
		}
	}
	return rawName;
}
const xlinkNS = 'http://www.w3.org/1999/xlink';
function patchAttr(el, key, value, isSVG, __unused_CC8B, isBoolean = isSpecialBooleanAttr(key)) {
	if (isSVG && key.startsWith('xlink:')) {
		if (value == null) {
			el.removeAttributeNS(xlinkNS, key.slice(6, key.length));
		} else {
			el.setAttributeNS(xlinkNS, key, value);
		}
	} else {
		if (value == null || isBoolean && !includeBooleanAttr(value)) {
			el.removeAttribute(key);
		} else {
			el.setAttribute(key, isBoolean ? '' : isSymbol(value) ? String(value) : value);
		}
	}
}
function patchDOMProp(el, key, value, __unused_F1D6, attrName) {
	if (key === 'innerHTML' || key === 'textContent') {
		if (value != null) {
			el[key] = key === 'innerHTML' ? unsafeToTrustedHTML(value) : value;
		}
		return;
	}
	const tag = el.tagName;
	if (key === 'value' && tag !== 'PROGRESS' && !tag.includes('-')) {
		const oldValue = tag === 'OPTION' ? el.getAttribute('value') || '' : el.value;
		const newValue = value == null ? el.type === 'checkbox' ? 'on' : '' : String(value);
		if (oldValue !== newValue || !('_value' in el)) {
			el.value = newValue;
		}
		if (value == null) {
			el.removeAttribute(key);
		}
		el._value = value;
		return;
	}
	let needRemove = false;
	if (value === '' || value == null) {
		const type = typeof el[key];
		if (type === 'boolean') {
			value = includeBooleanAttr(value);
		} else if (value == null && type === 'string') {
			value = '';
			needRemove = true;
		} else if (type === 'number') {
			value = 0;
			needRemove = true;
		}
	}
	try {
		el[key] = value;
	} catch {}
	needRemove && el.removeAttribute(attrName || key);
}
function addEventListener(el, event, handler, options) {
	el.addEventListener(event, handler, options);
}
function removeEventListener(el, event, handler, options) {
	el.removeEventListener(event, handler, options);
}
const veiKey = Symbol('_vei');
function patchEvent(el, rawName, __unused_DF07, nextValue, instance = null) {
	const invokers = el[veiKey] || (el[veiKey] = {});
	const existingInvoker = invokers[rawName];
	if (nextValue && existingInvoker) {
		existingInvoker.value = nextValue;
	} else {
		const [name, options] = parseName(rawName);
		if (nextValue) {
			const invoker = invokers[rawName] = createInvoker(nextValue, instance);
			addEventListener(el, name, invoker, options);
		} else if (existingInvoker) {
			removeEventListener(el, name, existingInvoker, options);
			invokers[rawName] = undefined;
		}
	}
}
const optionsModifierRE = /(?:Once|Passive|Capture)$/;
function parseName(name) {
	let options;
	if (optionsModifierRE.test(name)) {
		options = {};
		let m;
		while (m = name.match(optionsModifierRE)) {
			name = name.slice(0, name.length - m[0].length);
			options[m[0].toLowerCase()] = true;
		}
	}
	const event = name[2] === ':' ? name.slice(3) : hyphenate(name.slice(2));
	return [event, options];
}
let cachedNow = 0;
const p = Promise.resolve();
const getNow = () => cachedNow || (p.then(() => (cachedNow = 0, 0)), cachedNow = Date.now());
function createInvoker(initialValue, instance) {
	const invoker = (e) => {
		if (!e._vts) {
			e._vts = Date.now();
		} else if (e._vts <= invoker.attached) {
			return;
		}
		callWithAsyncErrorHandling(patchStopImmediatePropagation(e, invoker.value), instance, 5, [e]);
	};
	invoker.value = initialValue;
	invoker.attached = getNow();
	return invoker;
}
function patchStopImmediatePropagation(e, value) {
	if (isArray(value)) {
		const originalStop = e.stopImmediatePropagation;
		e.stopImmediatePropagation = () => {
			originalStop.call(e);
			e._stopped = true;
		};
		return value.map((fn) => (e2) => !e2._stopped && fn && fn(e2));
	} else {
		return value;
	}
}
const isNativeOn = (key) => key.charCodeAt(0) === 111 && key.charCodeAt(1) === 110 && key.charCodeAt(2) > 96 && key.charCodeAt(2) < 123;
const patchProp = (el, key, prevValue, nextValue, namespace, parentComponent) => {
	const isSVG = namespace === 'svg';
	if (key === 'class') {
		patchClass(el, nextValue, isSVG);
	} else if (key === 'style') {
		patchStyle(el, prevValue, nextValue);
	} else if (isOn(key)) {
		if (!isModelListener(key)) {
			patchEvent(el, key, 0, nextValue, parentComponent);
		}
	} else if (key[0] === '.' ? (key = key.slice(1), true) : key[0] === '^' ? (key = key.slice(1), false) : shouldSetAsProp(el, key, nextValue, isSVG)) {
		patchDOMProp(el, key, nextValue);
		if (!el.tagName.includes('-') && (key === 'value' || key === 'checked' || key === 'selected')) {
			patchAttr(el, key, nextValue, isSVG, 0, key !== 'value');
		}
	} else if (el._isVueCE && (/[A-Z]/.test(key) || !isString(nextValue))) {
		patchDOMProp(el, camelize(key), nextValue, 0, key);
	} else {
		if (key === 'true-value') {
			el._trueValue = nextValue;
		} else if (key === 'false-value') {
			el._falseValue = nextValue;
		}
		patchAttr(el, key, nextValue, isSVG);
	}
};
function shouldSetAsProp(el, key, value, isSVG) {
	if (isSVG) {
		if (key === 'innerHTML' || key === 'textContent') {
			return true;
		}
		if (key in el && isNativeOn(key) && isFunction(value)) {
			return true;
		}
		return false;
	}
	if (key === 'spellcheck' || key === 'draggable' || key === 'translate') {
		return false;
	}
	if (key === 'form') {
		return false;
	}
	if (key === 'list' && el.tagName === 'INPUT') {
		return false;
	}
	if (key === 'type' && el.tagName === 'TEXTAREA') {
		return false;
	}
	if (key === 'width' || key === 'height') {
		const tag = el.tagName;
		if (tag === 'IMG' || tag === 'VIDEO' || tag === 'CANVAS' || tag === 'SOURCE') {
			return false;
		}
	}
	if (isNativeOn(key) && isString(value)) {
		return false;
	}
	return key in el;
}
const rendererOptions = extend({ patchProp }, nodeOps);
function ensureRenderer() {
	return createRenderer(rendererOptions);
}
const createApp = (...args) => {
	const app = ensureRenderer().createApp(...args);
	const { mount } = app;
	app.mount = (containerOrSelector) => {
		const container = normalizeContainer(containerOrSelector);
		if (!container) return;
		const component = app._component;
		if (!isFunction(component) && !component.render && !component.template) {
			component.template = container.innerHTML;
		}
		if (container.nodeType === 1) {
			container.textContent = '';
		}
		const proxy = mount(container, false, resolveRootNamespace(container));
		if (container instanceof Element) {
			container.removeAttribute('v-cloak');
			container.setAttribute('data-v-app', '');
		}
		return proxy;
	};
	return app;
};
function resolveRootNamespace(container) {
	if (container instanceof SVGElement) {
		return 'svg';
	}
	if (typeof MathMLElement === 'function' && container instanceof MathMLElement) {
		return 'mathml';
	}
}
function normalizeContainer(container) {
	if (isString(container)) {
		const res = document.querySelector(container);
		return res;
	}
	return container;
}
const _export_sfc = (sfc, props) => {
	const target = sfc;
	for (const [, val] of props) {
		target.render = val;
	}
	return target;
};
const _sfc_main = {};
function _sfc_render() {
	return openBlock(), createElementBlock(0, 0, ' Hello World ');
}
const App = _export_sfc(_sfc_main, [[, _sfc_render]]);
createApp(App).mount('#app');
