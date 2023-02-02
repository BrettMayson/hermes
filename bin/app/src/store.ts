// store.ts
import { InjectionKey } from 'vue'
import { createStore, useStore as baseUseStore, Store } from 'vuex'

export interface State {
  received: boolean,
  firstTime: boolean,
  root: RootConfig,
}

export interface RootConfig {
  arma3folder: string | null,
  depotfolder: string | null,
}

// define injection key
export const key: InjectionKey<Store<State>> = Symbol()

export const store = createStore<State>({
  state: {
    received: false,
    firstTime: true,
    root: {
      arma3folder: null,
      depotfolder: null,
    },
  }
})

export function useStore() {
  return baseUseStore(key)
}
