import { Checkbox, Container, Group, NumberInput, Select, Tooltip } from "@mantine/core";
import { useForm } from "@mantine/form";
import { OrderMode, SettingsLiveScraper, StockMode } from "@api/types";
import { TooltipIcon } from "@components";
import { useTranslateEnums, useTranslateForms } from "@hooks/index";

export type LiveTradingPanelProps = {
  value: SettingsLiveScraper;
  onSubmit: (value: SettingsLiveScraper) => void;
}
export const LiveTradingPanel = ({ onSubmit, value }: LiveTradingPanelProps) => {

  // Translate general
  const useTranslateForm = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslateForms(`settings.tabs.live_trading.${key}`, { ...context }, i18Key)
  const useTranslateFormFields = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslateForm(`fields.${key}`, { ...context }, i18Key)
  const useTranslateStockMode = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslateEnums(`stock_mode.${key}`, { ...context }, i18Key)
  const useTranslateOrderMode = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslateEnums(`order_mode.${key}`, { ...context }, i18Key)
  // const useTranslateButtons = (key: string, context?: { [key: string]: any }, i18Key?: boolean) => useTranslateForm(`buttons.${key}`, { ...context }, i18Key)

  // Form
  const form = useForm({
    initialValues: value,
    validate: {},
  });

  return (
    <Container>
      <form onSubmit={form.onSubmit(() => { onSubmit(form.values) })}>
        <Group gap="md">
          <NumberInput
            label={useTranslateFormFields('volume_threshold.label')}
            placeholder={useTranslateFormFields('volume_threshold.placeholder')}
            value={form.values.stock_item.volume_threshold}
            onChange={(event) => form.setFieldValue('stock_item.volume_threshold', Number(event))}
            error={form.errors.volume_threshold && useTranslateFormFields('volume_threshold.error')}
            rightSection={<TooltipIcon label={useTranslateFormFields('volume_threshold.tooltip')} />}
            radius="md"
          />
          <NumberInput
            label={useTranslateFormFields('range_threshold.label')}
            placeholder={useTranslateFormFields('range_threshold.placeholder')}
            value={form.values.stock_item.range_threshold}
            onChange={(event) => form.setFieldValue('stock_item.range_threshold', Number(event))}
            error={form.errors.range_threshold && useTranslateFormFields('range_threshold.error')}
            rightSection={<TooltipIcon label={useTranslateFormFields('range_threshold.tooltip')} />}
            radius="md"
          />
          <NumberInput
            label={useTranslateFormFields('avg_price_cap.label')}
            placeholder={useTranslateFormFields('avg_price_cap.placeholder')}
            value={form.values.stock_item.avg_price_cap}
            onChange={(event) => form.setFieldValue('stock_item.avg_price_cap', Number(event))}
            error={form.errors.avg_price_cap && useTranslateFormFields('avg_price_cap.error')}
            rightSection={<TooltipIcon label={useTranslateFormFields('avg_price_cap.tooltip')} />}
            radius="md"
          />
          <NumberInput
            label={useTranslateFormFields('max_total_price_cap.label')}
            placeholder={useTranslateFormFields('max_total_price_cap.placeholder')}
            value={form.values.stock_item.max_total_price_cap}
            onChange={(event) => form.setFieldValue('stock_item.max_total_price_cap', Number(event))}
            error={form.errors.max_total_price_cap && useTranslateFormFields('max_total_price_cap.error')}
            rightSection={<TooltipIcon label={useTranslateFormFields('max_total_price_cap.tooltip')} />}
            radius="md"
          />
        </Group>
        <Group gap="md">
          <NumberInput
            label={useTranslateFormFields('price_shift_threshold.label')}
            placeholder={useTranslateFormFields('price_shift_threshold.placeholder')}
            value={form.values.stock_item.price_shift_threshold}
            onChange={(event) => form.setFieldValue('stock_item.price_shift_threshold', Number(event))}
            error={form.errors.price_shift_threshold && useTranslateFormFields('price_shift_threshold.error')}
            rightSection={<TooltipIcon label={useTranslateFormFields('price_shift_threshold.tooltip')} />}
            radius="md"
          />
          <NumberInput
            label={useTranslateFormFields('min_sma.label')}
            placeholder={useTranslateFormFields('min_sma.placeholder')}
            value={form.values.stock_item.min_sma}
            onChange={(event) => form.setFieldValue('stock_item.min_sma', Number(event))}
            error={form.errors.min_sma && useTranslateFormFields('min_sma.error')}
            rightSection={<TooltipIcon label={useTranslateFormFields('min_sma.tooltip')} />}
            radius="md"
          />
          <NumberInput
            label={useTranslateFormFields('min_profit.label')}
            placeholder={useTranslateFormFields('min_profit.placeholder')}
            value={form.values.stock_item.min_profit}
            onChange={(event) => form.setFieldValue('stock_item.min_profit', Number(event))}
            error={form.errors.min_profit && useTranslateFormFields('min_profit.error')}
            rightSection={<TooltipIcon label={useTranslateFormFields('min_profit.tooltip')} />}
            radius="md"
          />
        </Group>
        <Group gap="md">
          <Select
            label={useTranslateFormFields('stock_mode.label')}
            description={useTranslateFormFields(`stock_mode.description.${form.values.stock_mode}`)}
            placeholder={useTranslateFormFields('stock_mode.placeholder')}
            data={Object.values(StockMode).map((status) => {
              return { value: status, label: useTranslateStockMode(status) }
            })}
            value={form.values.stock_mode}
            onChange={(event) => form.setFieldValue('stock_mode', event as StockMode)}
            error={form.errors.stock_mode && useTranslateFormFields('stock_mode.error')}
            radius="md"
          />
          <Select
            label={useTranslateFormFields('order_mode.label')}
            description={useTranslateFormFields(`order_mode.description.${form.values.stock_item.order_mode}`)}
            placeholder={useTranslateFormFields('order_mode.placeholder')}
            data={Object.values(OrderMode).map((status) => {
              return { value: status, label: useTranslateOrderMode(status) }
            })}
            value={form.values.stock_item.order_mode}
            onChange={(event) => form.setFieldValue('stock_item.order_mode', event as OrderMode)}
            error={form.errors.order_mode && useTranslateFormFields('order_mode.error')}
            radius="md"
          />
        </Group>
        <Group gap={"md"} mt={25}>
          <Tooltip label={useTranslateFormFields('report_to_wfm.tooltip')}>
            <Checkbox
              label={useTranslateFormFields('report_to_wfm.label')}
              checked={form.values.stock_item.report_to_wfm}
              onChange={(event) => form.setFieldValue('stock_item.report_to_wfm', event.currentTarget.checked)}
              error={form.errors.report_to_wfm && useTranslateFormFields('report_to_wfm.error')}
            />
          </Tooltip>
          <Tooltip label={useTranslateFormFields('auto_trade.tooltip')} >
            <Checkbox
              label={useTranslateFormFields('auto_trade.label')}
              checked={form.values.stock_item.auto_trade}
              onChange={(event) => form.setFieldValue('stock_item.auto_trade', event.currentTarget.checked)}
              error={form.errors.auto_trade && useTranslateFormFields('auto_trade.error')}
            />
          </Tooltip>
          <Tooltip label={useTranslateFormFields('strict_whitelist.tooltip')}>
            <Checkbox
              label={useTranslateFormFields('strict_whitelist.label')}
              checked={form.values.stock_item.strict_whitelist}
              onChange={(event) => form.setFieldValue('stock_item.strict_whitelist', event.currentTarget.checked)}
              error={form.errors.strict_whitelist && useTranslateFormFields('strict_whitelist.error')}
            />
          </Tooltip>
        </Group>
      </form>
    </Container>
  );
};