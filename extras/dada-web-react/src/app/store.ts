import { configureStore, ThunkAction, Action } from "@reduxjs/toolkit";

import ideSlice from "../features/ide/ideSlice";

export const store = configureStore({
  reducer: {
    ide: ideSlice
  }
});

export type AppDispatch = typeof store.dispatch;
export type RootState = ReturnType<typeof store.getState>;
export type AppThunk<ReturnType = void> = ThunkAction<
  ReturnType,
  RootState,
  unknown,
  Action<string>
>;
