import {
  createContext,
  memo,
  PropsWithChildren,
  useCallback,
  useState,
} from 'react';

export interface ModalsStore {
  topModal: string | undefined;
  addModal(modal: string): void;
  removeModal(modal: string): void;
}

export const ModalsStoreContext = createContext<ModalsStore>(undefined as any);

export const ModalsProvider = memo<PropsWithChildren>(({ children }) => {
  const [modals, setModals] = useState<string[]>([]);

  const addModal = useCallback((modal: string) => {
    setModals((modals) => {
      return modals.indexOf(modal) === -1 ? [...modals, modal] : modals;
    });
  }, []);

  const removeModal = useCallback((modal: string) => {
    setModals((modals) => {
      const idx = modals.indexOf(modal);
      return idx !== -1
        ? [...modals.slice(0, idx), ...modals.slice(idx + 1)]
        : modals;
    });
  }, []);

  return (
    <ModalsStoreContext.Provider
      value={{
        topModal: modals.length > 0 ? modals[modals.length - 1] : undefined,
        addModal,
        removeModal,
      }}
    >
      {children}
    </ModalsStoreContext.Provider>
  );
});
