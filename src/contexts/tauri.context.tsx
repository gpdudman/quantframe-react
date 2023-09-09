import { createContext, useContext, useEffect, useState } from "react";
import { Wfm, Settings, TransactionEntryDto, InventoryEntryDto, StatisticDto, DeepPartial } from '$types/index';
import { isPermissionGranted, sendNotification } from '@tauri-apps/api/notification';
import api from "../api";
import { SplashScreen } from "../components/splashScreen";
import { useQuery } from "@tanstack/react-query";
import { notifications } from "@mantine/notifications";
import { OnTauriEvent, OnTauriUpdateDataEvent, getStatistic } from "@utils/index";
import { useTranslateContext } from "../hooks";




type TauriContextProps = {
  user: Wfm.UserDto | undefined;
  tradable_items: Wfm.ItemDto[];
  transactions: TransactionEntryDto[];
  orders: Wfm.OrderDto[];
  inventorys: InventoryEntryDto[];
  updateUser: (user: Partial<Wfm.UserDto>) => void;
  settings: Settings | undefined;
  statistics: StatisticDto | undefined,
  updateSettings: (user: DeepPartial<Settings>) => void;
  sendNotification: (title: string, body: string) => void;
}
type TauriContextProviderProps = {
  children: React.ReactNode;
}

export const TauriContext = createContext<TauriContextProps>({
  user: undefined,
  tradable_items: [],
  transactions: [],
  inventorys: [],
  orders: [],
  statistics: undefined,
  updateUser: () => { },
  settings: undefined,
  updateSettings: () => { },
  sendNotification: () => { },
});

export const useTauriContext = () => useContext(TauriContext);

export const TauriContextProvider = ({ children }: TauriContextProviderProps) => {
  const useTranslateTauri = (key: string, context?: { [key: string]: any }) => useTranslateContext(`tauri.${key}`, { ...context })
  const [user, setUser] = useState<Wfm.UserDto | undefined>(undefined);
  const [settings, setSettings] = useState<Settings | undefined>(undefined);
  const [tradable_items, setTradableItems] = useState<Wfm.ItemDto[]>([]);
  const [transactions, setTransactions] = useState<TransactionEntryDto[]>([]);
  const [inventorys, setInventorys] = useState<InventoryEntryDto[]>([]);
  const [statistics, setStatistics] = useState<StatisticDto | undefined>(undefined);
  const [orders, setOrders] = useState<Wfm.OrderDto[]>([]);

  const { isFetching } = useQuery({
    queryKey: ['validate'],
    queryFn: () => api.auth.validate(),
    onSuccess(data) {
      if (!data.valid) {
        notifications.show({
          title: useTranslateTauri("notifications.session_expired"),
          message: useTranslateTauri("notifications.session_expired_message"),
          color: 'red',
          autoClose: 5000,
        });
      } else
        setUser({ ...data.user })
      setSettings({ ...data.settings })
      setInventorys([...data.inventorys])
      setTransactions([...data.transactions])
      setOrders([...data.orders])
      console.log(data);
    },
  })

  useEffect(() => {
    if (!transactions) return;
    let statistics = getStatistic(transactions);
    setStatistics(statistics);
  }, [transactions]);


  const handleUpdateUser = (userData: Partial<Wfm.UserDto>) => {
    if (!user) return;
    setUser({ ...user, ...userData });
  }


  const handleUpdateSettings = async (settingsData: DeepPartial<Settings>) => {
    if (!settings) return;
    debugger
    const data = { ...settings, ...settingsData } as Settings;
    setSettings((a) => a = data);
    setSettings(await api.base.updatesettings(data as any)); // add 'as any' to avoid type checking
    notifications.show({
      title: useTranslateTauri("notifications.settings_updated"),
      message: useTranslateTauri("notifications.settings_updated_message"),
      color: 'green',
      autoClose: 5000,
    });
  }

  useEffect(() => {
    if (!settings) return;
    console.log("settings", settings);

  }, [settings]);

  const handleSendNotification = async (title: string, body: string) => {
    let permissionGranted = await isPermissionGranted();
    if (!permissionGranted) throw new Error("Permission not granted");
    if (permissionGranted) {
      sendNotification({ title: title, body: body });
    }
  }

  const handleUpdateOrders = (operation: string, data: Wfm.OrderDto | string) => {
    switch (operation) {
      case "create":
        {
          const order = data as Wfm.OrderDto;
          setOrders((inventorys) => [...inventorys, order]);
        }
        break;
      case "update":
        {
          const order = data as Wfm.OrderDto;
          setOrders((inventorys) => [...inventorys.filter((item) => item.id !== order.id), order]);
        }
        break;
      case "delete":
        {
          const order_id = data as string;
          setOrders((inventorys) => [...inventorys.filter((item) => item.id !== order_id)]);
        }
        break;
    }
  }

  const handleUpdateInventory = (operation: string, data: InventoryEntryDto) => {
    switch (operation) {
      case "create":
        setInventorys((inventorys) => [...inventorys, data]);
        break;
      case "update":
        setInventorys((inventorys) => [...inventorys.filter((item) => item.id !== data.id), data]);
        break;
      case "delete":
        setInventorys((inventorys) => [...inventorys.filter((item) => item.id !== data.id)]);
        break;
    }
  }

  const handleUpdateTransaction = (operation: string, data: TransactionEntryDto) => {
    switch (operation) {
      case "create":
        setTransactions((transactions) => [...transactions, data]);
        break;
      case "update":
        setTransactions((transactions) => [...transactions.filter((item) => item.id !== data.id), data]);
        break;
      case "delete":
        setTransactions((transactions) => [...transactions.filter((item) => item.id !== data.id)]);
        break;
    }
  }

  useEffect(() => {
    OnTauriEvent("update_tradable_items", (data: Wfm.ItemDto[]) => {
      setTradableItems(data);
    });
    OnTauriUpdateDataEvent<InventoryEntryDto>("inventorys", ({ data, operation }) => handleUpdateInventory(operation, data));
    OnTauriUpdateDataEvent<TransactionEntryDto>("transactions", ({ data, operation }) => handleUpdateTransaction(operation, data));
    OnTauriUpdateDataEvent<Wfm.OrderDto | string>("orders", ({ data, operation }) => handleUpdateOrders(operation, data));
    return () => { }
  }, []);

  return (
    <TauriContext.Provider value={{ user, orders, statistics, transactions, inventorys, tradable_items, updateUser: handleUpdateUser, settings, updateSettings: handleUpdateSettings, sendNotification: handleSendNotification }}>
      <SplashScreen opened={isFetching} />
      {children}
    </TauriContext.Provider>
  )
}