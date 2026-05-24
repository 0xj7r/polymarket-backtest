#!/usr/bin/env bash
# Launch one EC2 runner for a train-once, portfolio-grid backtest.
#
# The instance:
#   1. syncs this repo from S3,
#   2. builds pm-app,
#   3. extracts/trains the meta-calibrator once on the first N markets,
#   4. reuses the frozen snapshot for every clip/gross sizing config,
#   5. uploads summaries, market rows, logs, and model artifacts to S3,
#   6. terminates itself.
#
# Usage:
#   AWS_PROFILE=visumlabs ./scripts/launch_ec2_portfolio_grid.sh \
#     --start-date 2026-02-12 --end-date 2026-05-20 \
#     --train-markets 20000 \
#     --clip-fractions 0.003,0.005,0.0075,0.01 \
#     --gross-caps 500,750,1000
set -euo pipefail

REGION="${AWS_REGION:-us-east-1}"
INSTANCE_TYPE="${INSTANCE_TYPE:-c7i.4xlarge}"
RESULTS_BUCKET="${RESULTS_BUCKET:-pm-research-backtest-prod}"
SOURCE_BUCKET="${SOURCE_BUCKET:-pm-research-backtest-prod}"
SOURCE_PREFIX="${SOURCE_PREFIX:-source/polymarket-backtest}"
MARKETS_PARQUET_BUCKET="${MARKETS_PARQUET_BUCKET:-pm-research-backtest-prod}"
MARKETS_PARQUET_KEY="${MARKETS_PARQUET_KEY:-artifacts/markets-full.parquet}"
INSTANCE_PROFILE="${INSTANCE_PROFILE:-instanceRole}"
KEY_NAME="${KEY_NAME:-whale-pair-use1}"
SECURITY_GROUP_ID="${SECURITY_GROUP_ID:-sg-0714c4165723a894a}"
SUBNET_ID="${SUBNET_ID:-subnet-0c16e9b7f39d97feb}"
ROOT_VOLUME_GB="${ROOT_VOLUME_GB:-250}"
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-portfolio-grid-$$"

MARKETS_KEY=""
START_DATE=""
END_DATE=""
STRATEGIES="bonereaper_v2"
STARTING_CASH="1000"
MAX_CLIP="100"
MAX_ORDER_CLIP_MULTIPLIER="6.0"
KELLY="0.25"
CLIP_DRAWDOWN_SOFT_PCT="1.0"
CLIP_DRAWDOWN_HARD_PCT="1.0"
BR2_DISABLE_INTERNAL_MODEL_GATES="0"
BR2_MIN_COMPOSITE_DIRECTION="0.10"
BR2_EARLY_CLIP_FRAC="0.00"
BR2_MID_CLIP_FRAC="0.00"
BR2_LATE_CLIP_FRAC="1.0"
BR2_LATE_MAX_FIRES="3"
BR2_LATE_CONFIRM_MIN_MODEL_CONFIDENCE="0.58"
BR2_LATE_CONFIRM_MAX_MODEL_RISK="0.80"
BR2_LATE_CONFIRM_MIN_MODEL_SIDE_P="0.58"
BR2_LATE_CONFIRM_MIN_MODEL_EDGE="0.00"
BR2_LATE_CONFIRM_MAX_WHIPSAW_SCORE="0.85"
BR2_HIGH_SKEW_CLIP_FRAC="0.60"
BR2_HIGH_SKEW_MAX_CLIPS="5"
BR2_HIGH_SKEW_MAX_WHIPSAW_SCORE="0.75"
BR2_LATE_FAVOURITE_START_SECS="180.0"
BR2_LATE_FAVOURITE_THRESHOLD="0.22"
BR2_LATE_FAVOURITE_CLIP_FRAC="1.00"
BR2_LATE_FAVOURITE_MAX_CLIPS="12"
BR2_LATE_FAVOURITE_SWEEP_DEPTH="7"
BR2_LATE_FAVOURITE_MIN_MODEL_CONFIDENCE="0.68"
BR2_LATE_FAVOURITE_MAX_MODEL_RISK="0.72"
BR2_LATE_FAVOURITE_MIN_MODEL_SIDE_P="0.62"
BR2_LATE_FAVOURITE_MIN_MODEL_EDGE="0.00"
BR2_LATE_FAVOURITE_MAX_WHIPSAW_SCORE="0.75"
ENFORCE_MODEL_GATE="true"
MODEL_GATE_MIN_CONFIDENCE="0.68"
MODEL_GATE_MAX_RISK="0.72"
MODEL_GATE_MIN_EDGE="0.00"
TRAIN_MARKETS="2880"
META_EPOCHS="24"
META_LEARNING_RATE="0.04"
META_L2="0.001"
META_WEIGHT_CLIP="1.50"
META_MAX_FIT_SAMPLES="120000"
META_MAX_VALIDATION_SAMPLES="60000"
META_MAX_SAMPLES_PER_MARKET="64"
META_MAX_OOS_EVALUATION_SAMPLES="120000"
DISABLE_META_CALIBRATION="0"
CLIP_FRACTIONS="0.015,0.02,0.03"
GROSS_CAPS="250,500,750"
SPOT_SYMBOL="BTCUSDT"
MAX_CONCURRENT_FETCHES="32"
USE_LOCAL_CACHE="0"
LOCAL_CACHE_DIR="/opt/pm/cache"
PREP_CACHE_MAX_CONCURRENT="128"
DISCOVERY_MAX_CONCURRENT="4"
PORTFOLIO_CHECKPOINT_EVERY_MARKETS="250"
SYNC_SOURCE="1"
SNAPSHOT_S3_URI=""
META_TRAINING_SAMPLES_CACHE_S3_URI=""

while [ $# -gt 0 ]; do
    case "$1" in
        --markets-key) MARKETS_KEY="$2"; shift 2 ;;
        --start-date) START_DATE="$2"; shift 2 ;;
        --end-date) END_DATE="$2"; shift 2 ;;
        --strategies) STRATEGIES="$2"; shift 2 ;;
        --starting-cash) STARTING_CASH="$2"; shift 2 ;;
        --max-clip) MAX_CLIP="$2"; shift 2 ;;
        --max-order-clip-multiplier) MAX_ORDER_CLIP_MULTIPLIER="$2"; shift 2 ;;
        --kelly) KELLY="$2"; shift 2 ;;
        --clip-drawdown-soft-pct) CLIP_DRAWDOWN_SOFT_PCT="$2"; shift 2 ;;
        --clip-drawdown-hard-pct) CLIP_DRAWDOWN_HARD_PCT="$2"; shift 2 ;;
        --br2-disable-internal-model-gates) BR2_DISABLE_INTERNAL_MODEL_GATES="1"; shift ;;
        --br2-min-composite-direction) BR2_MIN_COMPOSITE_DIRECTION="$2"; shift 2 ;;
        --br2-early-clip-frac) BR2_EARLY_CLIP_FRAC="$2"; shift 2 ;;
        --br2-mid-clip-frac) BR2_MID_CLIP_FRAC="$2"; shift 2 ;;
        --br2-late-clip-frac) BR2_LATE_CLIP_FRAC="$2"; shift 2 ;;
        --br2-late-max-fires) BR2_LATE_MAX_FIRES="$2"; shift 2 ;;
        --br2-late-confirm-min-model-confidence) BR2_LATE_CONFIRM_MIN_MODEL_CONFIDENCE="$2"; shift 2 ;;
        --br2-late-confirm-max-model-risk) BR2_LATE_CONFIRM_MAX_MODEL_RISK="$2"; shift 2 ;;
        --br2-late-confirm-min-model-side-p) BR2_LATE_CONFIRM_MIN_MODEL_SIDE_P="$2"; shift 2 ;;
        --br2-late-confirm-min-model-edge) BR2_LATE_CONFIRM_MIN_MODEL_EDGE="$2"; shift 2 ;;
        --br2-late-confirm-max-whipsaw-score) BR2_LATE_CONFIRM_MAX_WHIPSAW_SCORE="$2"; shift 2 ;;
        --br2-high-skew-clip-frac) BR2_HIGH_SKEW_CLIP_FRAC="$2"; shift 2 ;;
        --br2-high-skew-max-clips) BR2_HIGH_SKEW_MAX_CLIPS="$2"; shift 2 ;;
        --br2-high-skew-max-whipsaw-score) BR2_HIGH_SKEW_MAX_WHIPSAW_SCORE="$2"; shift 2 ;;
        --br2-late-favourite-start-secs) BR2_LATE_FAVOURITE_START_SECS="$2"; shift 2 ;;
        --br2-late-favourite-threshold) BR2_LATE_FAVOURITE_THRESHOLD="$2"; shift 2 ;;
        --br2-late-favourite-clip-frac) BR2_LATE_FAVOURITE_CLIP_FRAC="$2"; shift 2 ;;
        --br2-late-favourite-max-clips) BR2_LATE_FAVOURITE_MAX_CLIPS="$2"; shift 2 ;;
        --br2-late-favourite-sweep-depth) BR2_LATE_FAVOURITE_SWEEP_DEPTH="$2"; shift 2 ;;
        --br2-late-favourite-min-model-confidence) BR2_LATE_FAVOURITE_MIN_MODEL_CONFIDENCE="$2"; shift 2 ;;
        --br2-late-favourite-max-model-risk) BR2_LATE_FAVOURITE_MAX_MODEL_RISK="$2"; shift 2 ;;
        --br2-late-favourite-min-model-side-p) BR2_LATE_FAVOURITE_MIN_MODEL_SIDE_P="$2"; shift 2 ;;
        --br2-late-favourite-min-model-edge) BR2_LATE_FAVOURITE_MIN_MODEL_EDGE="$2"; shift 2 ;;
        --br2-late-favourite-max-whipsaw-score) BR2_LATE_FAVOURITE_MAX_WHIPSAW_SCORE="$2"; shift 2 ;;
        --enforce-model-gate) ENFORCE_MODEL_GATE="$2"; shift 2 ;;
        --model-gate-min-confidence) MODEL_GATE_MIN_CONFIDENCE="$2"; shift 2 ;;
        --model-gate-max-risk) MODEL_GATE_MAX_RISK="$2"; shift 2 ;;
        --model-gate-min-edge) MODEL_GATE_MIN_EDGE="$2"; shift 2 ;;
        --train-markets) TRAIN_MARKETS="$2"; shift 2 ;;
        --meta-epochs) META_EPOCHS="$2"; shift 2 ;;
        --meta-learning-rate) META_LEARNING_RATE="$2"; shift 2 ;;
        --meta-l2) META_L2="$2"; shift 2 ;;
        --meta-weight-clip) META_WEIGHT_CLIP="$2"; shift 2 ;;
        --meta-max-fit-samples) META_MAX_FIT_SAMPLES="$2"; shift 2 ;;
        --meta-max-validation-samples) META_MAX_VALIDATION_SAMPLES="$2"; shift 2 ;;
        --meta-max-samples-per-market) META_MAX_SAMPLES_PER_MARKET="$2"; shift 2 ;;
        --meta-max-oos-evaluation-samples) META_MAX_OOS_EVALUATION_SAMPLES="$2"; shift 2 ;;
        --disable-meta-calibration) DISABLE_META_CALIBRATION="1"; shift ;;
        --clip-fractions) CLIP_FRACTIONS="$2"; shift 2 ;;
        --gross-caps) GROSS_CAPS="$2"; shift 2 ;;
        --spot-symbol) SPOT_SYMBOL="$2"; shift 2 ;;
        --max-concurrent-fetches) MAX_CONCURRENT_FETCHES="$2"; shift 2 ;;
        --use-local-cache) USE_LOCAL_CACHE="1"; shift ;;
        --local-cache-dir) LOCAL_CACHE_DIR="$2"; shift 2 ;;
        --prep-cache-max-concurrent) PREP_CACHE_MAX_CONCURRENT="$2"; shift 2 ;;
        --discovery-max-concurrent) DISCOVERY_MAX_CONCURRENT="$2"; shift 2 ;;
        --portfolio-checkpoint-every-markets) PORTFOLIO_CHECKPOINT_EVERY_MARKETS="$2"; shift 2 ;;
        --instance-type) INSTANCE_TYPE="$2"; shift 2 ;;
        --instance-profile) INSTANCE_PROFILE="$2"; shift 2 ;;
        --markets-parquet-bucket) MARKETS_PARQUET_BUCKET="$2"; shift 2 ;;
        --markets-parquet-key) MARKETS_PARQUET_KEY="$2"; shift 2 ;;
        --key-name) KEY_NAME="$2"; shift 2 ;;
        --security-group-id) SECURITY_GROUP_ID="$2"; shift 2 ;;
        --subnet-id) SUBNET_ID="$2"; shift 2 ;;
        --root-volume-gb) ROOT_VOLUME_GB="$2"; shift 2 ;;
        --snapshot-s3-uri) SNAPSHOT_S3_URI="$2"; shift 2 ;;
        --meta-training-samples-cache-s3-uri) META_TRAINING_SAMPLES_CACHE_S3_URI="$2"; shift 2 ;;
        --no-source-sync) SYNC_SOURCE="0"; shift ;;
        *) echo "unknown arg: $1" >&2; exit 1 ;;
    esac
done

if [ -z "$MARKETS_KEY" ] && { [ -z "$START_DATE" ] || [ -z "$END_DATE" ]; }; then
    echo "either --markets-key or both --start-date and --end-date are required" >&2
    exit 1
fi

if [ "$SYNC_SOURCE" = "1" ]; then
    echo "Syncing source to s3://${SOURCE_BUCKET}/${SOURCE_PREFIX}/"
    aws s3 rm "s3://${SOURCE_BUCKET}/${SOURCE_PREFIX}/" --recursive --quiet
    for file in Cargo.toml Cargo.lock rust-toolchain.toml README.md .gitignore; do
        [ -f "$file" ] && aws s3 cp "$file" "s3://${SOURCE_BUCKET}/${SOURCE_PREFIX}/${file}" --quiet
    done
    aws s3 sync crates "s3://${SOURCE_BUCKET}/${SOURCE_PREFIX}/crates/" --delete --quiet
    aws s3 sync scripts "s3://${SOURCE_BUCKET}/${SOURCE_PREFIX}/scripts/" --delete --quiet
    aws s3 sync configs "s3://${SOURCE_BUCKET}/${SOURCE_PREFIX}/configs/" --delete --quiet
    [ -d docs ] && aws s3 sync docs "s3://${SOURCE_BUCKET}/${SOURCE_PREFIX}/docs/" --delete --quiet
fi

AMI=$(aws ssm get-parameter \
    --region "$REGION" \
    --name /aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64 \
    --query Parameter.Value --output text)

echo "AMI: $AMI"
echo "Run ID: $RUN_ID"

USER_DATA=$(cat <<EOF
#!/bin/bash
set -euo pipefail
exec > >(tee -a /var/log/pm-bootstrap.log) 2>&1
echo "[\$(date -u)] starting pm portfolio grid run_id=${RUN_ID}"

dnf install -y git gcc gcc-c++ make openssl-devel pkgconf-pkg-config cmake clang jq
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y --default-toolchain 1.95
export HOME=/root
source /root/.cargo/env

mkdir -p /opt/pm/artifacts /opt/pm/results
aws s3 sync "s3://${SOURCE_BUCKET}/${SOURCE_PREFIX}/" /opt/pm/ \
    --exclude "target/*" --exclude "data/*" --exclude ".git/*"
cd /opt/pm
cargo build --release -p pm-app

if [ -n "${MARKETS_KEY}" ]; then
  aws s3 cp "s3://${SOURCE_BUCKET}/markets/${MARKETS_KEY}" /opt/pm/markets.jsonl
else
  aws s3 cp "s3://${MARKETS_PARQUET_BUCKET}/${MARKETS_PARQUET_KEY}" /opt/pm/markets-full.parquet
  PM_TELONEX_REGION="${REGION}" ./target/release/pm-app discover-markets-parquet \\
    --markets-parquet /opt/pm/markets-full.parquet \\
    --start-date "${START_DATE}" \\
    --end-date "${END_DATE}" \\
    --slug-prefix btc-updown-5m- \\
    --require-book-s3 \\
    --out /opt/pm/markets.jsonl
  aws s3 cp /opt/pm/markets.jsonl "s3://${RESULTS_BUCKET}/results/${RUN_ID}/artifacts/markets.jsonl"
fi

SNAPSHOT_IN=""
if [ -n "${SNAPSHOT_S3_URI}" ]; then
  SNAPSHOT_IN="/opt/pm/artifacts/meta-calibrator-snapshot-in.json"
  aws s3 cp "${SNAPSHOT_S3_URI}" "\${SNAPSHOT_IN}"
fi

if [ -n "${META_TRAINING_SAMPLES_CACHE_S3_URI}" ]; then
  aws s3 cp "${META_TRAINING_SAMPLES_CACHE_S3_URI}" /opt/pm/artifacts/meta-training-samples.json
fi

LOCAL_CACHE_ARGS=()
if [ "${USE_LOCAL_CACHE}" = "1" ]; then
  echo "[\$(date -u)] prewarming local Telonex cache at ${LOCAL_CACHE_DIR}"
  PM_TELONEX_REGION="${REGION}" ./target/release/pm-app prep-cache \\
    --markets /opt/pm/markets.jsonl \\
    --cache-dir "${LOCAL_CACHE_DIR}" \\
    --spot-symbol "${SPOT_SYMBOL}" \\
    --max-concurrent "${PREP_CACHE_MAX_CONCURRENT}"
  LOCAL_CACHE_ARGS=(--local-cache-dir "${LOCAL_CACHE_DIR}")
fi

MODEL_GATE_ARGS=()
if [ "${ENFORCE_MODEL_GATE}" != "true" ]; then
  MODEL_GATE_ARGS=(--disable-model-gate)
fi
BR2_INTERNAL_MODEL_GATE_ARGS=()
if [ "${BR2_DISABLE_INTERNAL_MODEL_GATES}" = "1" ]; then
  BR2_INTERNAL_MODEL_GATE_ARGS=(--br2-disable-internal-model-gates)
fi

IFS=',' read -r -a CLIPS <<< "${CLIP_FRACTIONS}"
IFS=',' read -r -a GROSS_CAPS <<< "${GROSS_CAPS}"
FIRST=1

for CLIP_FRAC in "\${CLIPS[@]}"; do
  for GROSS_CAP in "\${GROSS_CAPS[@]}"; do
    LABEL="clip_\${CLIP_FRAC//./p}_gross_\${GROSS_CAP//./p}"
    OUT_DIR="/opt/pm/results/\${LABEL}"
    mkdir -p "\${OUT_DIR}"
    echo "[\$(date -u)] running \${LABEL}"

    EXTRA_MODEL_ARGS=()
    if [ "${DISABLE_META_CALIBRATION}" = "1" ]; then
      EXTRA_MODEL_ARGS=(
        --disable-meta-calibration
      )
    elif [ -n "\${SNAPSHOT_IN}" ]; then
      EXTRA_MODEL_ARGS=(
        --meta-calibrator-snapshot-in "\${SNAPSHOT_IN}"
      )
    elif [ "\${FIRST}" = "1" ]; then
      EXTRA_MODEL_ARGS=(
        --meta-training-samples-cache /opt/pm/artifacts/meta-training-samples.json
        --meta-calibrator-snapshot-out /opt/pm/artifacts/meta-calibrator-snapshot.json
      )
      FIRST=0
    else
      EXTRA_MODEL_ARGS=(
        --meta-calibrator-snapshot-in /opt/pm/artifacts/meta-calibrator-snapshot.json
      )
    fi

    (
      while true; do
        sleep 180
        [ -f "\${OUT_DIR}/markets.jsonl" ] && aws s3 cp "\${OUT_DIR}/markets.jsonl" "s3://${RESULTS_BUCKET}/results/${RUN_ID}/\${LABEL}/markets.jsonl" || true
        [ -f "\${OUT_DIR}/summary.json" ] && aws s3 cp "\${OUT_DIR}/summary.json" "s3://${RESULTS_BUCKET}/results/${RUN_ID}/\${LABEL}/summary.json" || true
      done
    ) &
    UPLOADER_PID="\$!"

    set +e
    PM_TELONEX_REGION="${REGION}" ./target/release/pm-app walk-forward \\
      --markets /opt/pm/markets.jsonl \\
      --strategies "${STRATEGIES}" \\
      --starting-cash "${STARTING_CASH}" \\
      --max-clip-usdc "${MAX_CLIP}" \\
      --max-order-clip-multiplier "${MAX_ORDER_CLIP_MULTIPLIER}" \\
      --max-per-market-exposure-usdc "\${GROSS_CAP}" \\
      --kelly-fraction "${KELLY}" \\
      --spot-symbol "${SPOT_SYMBOL}" \\
      --use-outcome-label \\
      --portfolio-mode \\
      --clip-fraction-of-equity "\${CLIP_FRAC}" \\
      --clip-drawdown-soft-pct "${CLIP_DRAWDOWN_SOFT_PCT}" \\
      --clip-drawdown-hard-pct "${CLIP_DRAWDOWN_HARD_PCT}" \\
      "\${BR2_INTERNAL_MODEL_GATE_ARGS[@]}" \\
      --br2-min-composite-direction "${BR2_MIN_COMPOSITE_DIRECTION}" \\
      --br2-early-clip-frac "${BR2_EARLY_CLIP_FRAC}" \\
      --br2-mid-clip-frac "${BR2_MID_CLIP_FRAC}" \\
      --br2-late-clip-frac "${BR2_LATE_CLIP_FRAC}" \\
      --br2-late-max-fires "${BR2_LATE_MAX_FIRES}" \\
      --br2-late-confirm-min-model-confidence "${BR2_LATE_CONFIRM_MIN_MODEL_CONFIDENCE}" \\
      --br2-late-confirm-max-model-risk "${BR2_LATE_CONFIRM_MAX_MODEL_RISK}" \\
      --br2-late-confirm-min-model-side-p "${BR2_LATE_CONFIRM_MIN_MODEL_SIDE_P}" \\
      --br2-late-confirm-min-model-edge "${BR2_LATE_CONFIRM_MIN_MODEL_EDGE}" \\
      --br2-late-confirm-max-whipsaw-score "${BR2_LATE_CONFIRM_MAX_WHIPSAW_SCORE}" \\
      --br2-high-skew-clip-frac "${BR2_HIGH_SKEW_CLIP_FRAC}" \\
      --br2-high-skew-max-clips "${BR2_HIGH_SKEW_MAX_CLIPS}" \\
      --br2-high-skew-max-whipsaw-score "${BR2_HIGH_SKEW_MAX_WHIPSAW_SCORE}" \\
      --br2-late-favourite-start-secs "${BR2_LATE_FAVOURITE_START_SECS}" \\
      --br2-late-favourite-threshold "${BR2_LATE_FAVOURITE_THRESHOLD}" \\
      --br2-late-favourite-clip-frac "${BR2_LATE_FAVOURITE_CLIP_FRAC}" \\
      --br2-late-favourite-max-clips "${BR2_LATE_FAVOURITE_MAX_CLIPS}" \\
      --br2-late-favourite-sweep-depth "${BR2_LATE_FAVOURITE_SWEEP_DEPTH}" \\
      --br2-late-favourite-min-model-confidence "${BR2_LATE_FAVOURITE_MIN_MODEL_CONFIDENCE}" \\
      --br2-late-favourite-max-model-risk "${BR2_LATE_FAVOURITE_MAX_MODEL_RISK}" \\
      --br2-late-favourite-min-model-side-p "${BR2_LATE_FAVOURITE_MIN_MODEL_SIDE_P}" \\
      --br2-late-favourite-min-model-edge "${BR2_LATE_FAVOURITE_MIN_MODEL_EDGE}" \\
      --br2-late-favourite-max-whipsaw-score "${BR2_LATE_FAVOURITE_MAX_WHIPSAW_SCORE}" \\
      "\${MODEL_GATE_ARGS[@]}" \\
      --model-gate-min-confidence "${MODEL_GATE_MIN_CONFIDENCE}" \\
      --model-gate-max-risk "${MODEL_GATE_MAX_RISK}" \\
      --model-gate-min-edge "${MODEL_GATE_MIN_EDGE}" \\
      --min-train-markets "${TRAIN_MARKETS}" \\
      --meta-epochs "${META_EPOCHS}" \\
      --meta-learning-rate "${META_LEARNING_RATE}" \\
      --meta-l2 "${META_L2}" \\
      --meta-weight-clip "${META_WEIGHT_CLIP}" \\
      --meta-max-fit-samples "${META_MAX_FIT_SAMPLES}" \\
      --meta-max-validation-samples "${META_MAX_VALIDATION_SAMPLES}" \\
      --meta-max-samples-per-market "${META_MAX_SAMPLES_PER_MARKET}" \\
      --meta-max-oos-evaluation-samples "${META_MAX_OOS_EVALUATION_SAMPLES}" \\
      --max-concurrent-fetches "${MAX_CONCURRENT_FETCHES}" \\
      --portfolio-checkpoint-every-markets "${PORTFOLIO_CHECKPOINT_EVERY_MARKETS}" \\
      "\${LOCAL_CACHE_ARGS[@]}" \\
      "\${EXTRA_MODEL_ARGS[@]}" \\
      --out-markets "\${OUT_DIR}/markets.jsonl" \\
      --out-summary "\${OUT_DIR}/summary.json" \\
      2>&1 | tee "\${OUT_DIR}/run.log"
    RUN_STATUS="\${PIPESTATUS[0]}"
    set -e
    kill "\${UPLOADER_PID}" || true
    wait "\${UPLOADER_PID}" || true
    if [ "\${RUN_STATUS}" -ne 0 ]; then
      aws s3 cp "\${OUT_DIR}/run.log" "s3://${RESULTS_BUCKET}/results/${RUN_ID}/\${LABEL}/run.log" || true
      exit "\${RUN_STATUS}"
    fi

    aws s3 cp "\${OUT_DIR}/markets.jsonl" "s3://${RESULTS_BUCKET}/results/${RUN_ID}/\${LABEL}/markets.jsonl"
    aws s3 cp "\${OUT_DIR}/summary.json" "s3://${RESULTS_BUCKET}/results/${RUN_ID}/\${LABEL}/summary.json"
    aws s3 cp "\${OUT_DIR}/run.log" "s3://${RESULTS_BUCKET}/results/${RUN_ID}/\${LABEL}/run.log"
  done
done

if [ "${DISABLE_META_CALIBRATION}" != "1" ]; then
  aws s3 cp /opt/pm/artifacts/meta-training-samples.json "s3://${RESULTS_BUCKET}/results/${RUN_ID}/artifacts/meta-training-samples.json" || true
  aws s3 cp /opt/pm/artifacts/meta-calibrator-snapshot.json "s3://${RESULTS_BUCKET}/results/${RUN_ID}/artifacts/meta-calibrator-snapshot.json" || true
fi
aws s3 cp /var/log/pm-bootstrap.log "s3://${RESULTS_BUCKET}/results/${RUN_ID}/bootstrap.log"

echo "[\$(date -u)] portfolio grid complete; shutting down"
shutdown -h now
EOF
)

INSTANCE_ID=$(aws ec2 run-instances \
    --region "$REGION" \
    --image-id "$AMI" \
    --instance-type "$INSTANCE_TYPE" \
    --key-name "$KEY_NAME" \
    --subnet-id "$SUBNET_ID" \
    --security-group-ids "$SECURITY_GROUP_ID" \
    --iam-instance-profile "Name=$INSTANCE_PROFILE" \
    --instance-initiated-shutdown-behavior terminate \
    --block-device-mappings "DeviceName=/dev/xvda,Ebs={VolumeSize=${ROOT_VOLUME_GB},VolumeType=gp3,DeleteOnTermination=true}" \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=pm-backtest-${RUN_ID}},{Key=run_id,Value=${RUN_ID}}]" \
    --user-data "$USER_DATA" \
    --query 'Instances[0].InstanceId' --output text)

echo "Launched: $INSTANCE_ID"
echo "Results: s3://${RESULTS_BUCKET}/results/${RUN_ID}/"
echo
echo "Watch:"
echo "  aws s3 ls s3://${RESULTS_BUCKET}/results/${RUN_ID}/ --recursive"
echo "  aws ec2 describe-instance-status --instance-ids ${INSTANCE_ID} --region ${REGION}"
